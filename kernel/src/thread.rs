//! Kernel threads and the round-robin scheduler. Every timer tick hands the
//! CPU to the next ready thread, and the idle thread runs only when no
//! thread is ready. The boot thread runs on the loader's stack; every other
//! thread has a stack from `paging::map_stack`. A user thread is a kernel
//! thread that owns an address space and has left for user code in it;
//! every switch loads the next thread's space, or the kernel's tables.
//!
//! A thread blocks with [`park`] or [`park_until`] until [`unpark`] names
//! it or its deadline passes. Each tick readies the blocked threads whose
//! deadline has passed before it switches.
//!
//! The scheduler's lock is only ever held with interrupts off, so the
//! timer's handler never finds it held, and nothing allocates or frees
//! while holding it: a tick can preempt a thread that holds the heap's lock,
//! and the heap would then spin with interrupts off for good. `spawn`
//! reserves room in the queues for every thread, so moving one never
//! allocates.

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::mem;
use core::ops::Range;
use core::sync::atomic::{AtomicU64, Ordering};

use spin::Mutex;
use thisos_hal::{AddressSpace as _, Cpu, Platform};

use crate::{Arch, frames, paging, timer};

type Context = <Arch as Platform>::Context;

/// The size of every thread's stack but the boot thread's. aarch64's trap
/// entries tell an overflow by bit 16 of the stack pointer, which needs
/// 64 KiB stacks aligned to 128 KiB, as `map_stack` aligns them.
pub const STACK_SIZE: usize = 64 << 10;

struct Thread {
    /// Names it to [`unpark`]. Never reused, so a stale id unparks nothing.
    id: u64,
    /// Where it stopped, while it isn't running.
    context: Option<Context>,
    /// Its stack, or `None` for the boot thread, whose stack is the
    /// loader's and is never freed.
    stack: Option<Range<u64>>,
    /// What it runs, until it starts.
    run: Option<Box<dyn FnOnce() + Send>>,
    /// The address space it runs user code in, or `None` for a kernel
    /// thread.
    space: Option<paging::AddressSpace>,
    /// The tick at which it stops being blocked, while it is.
    deadline: Option<u64>,
    /// Why its next park returns, once something has woken it: set by an
    /// unpark or a deadline that comes while it is blocked, and by an
    /// unpark that comes before it parks.
    wake: Option<Wake>,
}

impl Thread {
    fn new(
        context: Option<Context>,
        stack: Option<Range<u64>>,
        run: Option<Box<dyn FnOnce() + Send>>,
        space: Option<paging::AddressSpace>,
    ) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            context,
            stack,
            run,
            space,
            deadline: None,
            wake: None,
        }
    }
}

/// Why a park returned.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wake {
    Unparked,
    TimedOut,
}

/// Where the thread that switches away goes.
#[derive(Clone, Copy)]
enum Place {
    Ready,
    Blocked,
    Dead,
    Idle,
}

struct Scheduler {
    running: Option<Box<Thread>>,
    ready: VecDeque<Box<Thread>>,
    /// Parked threads, in order of id.
    #[expect(
        clippy::vec_box,
        reason = "threads move between this and the ready queue as boxes, which never allocates"
    )]
    blocked: Vec<Box<Thread>>,
    /// The idle thread, or `None` while it runs.
    idle: Option<Box<Thread>>,
    /// Threads that exited, until `reap` frees them.
    #[expect(
        clippy::vec_box,
        reason = "unboxing a thread under the scheduler's lock would free the box there"
    )]
    dead: Vec<Box<Thread>>,
    /// The thread that just switched away, without its context, until the
    /// thread it switched to puts it in its place with that context.
    leaving: Option<(Box<Thread>, Place)>,
    /// Threads that aren't freed yet, the boot thread included and the idle
    /// thread not: the room `ready`, `blocked` and `dead` each need.
    threads: usize,
}

static SCHEDULER: Mutex<Option<Scheduler>> = Mutex::new(None);

/// Runs `f` on the scheduler with interrupts off. `f` must not allocate or
/// free.
fn with<T>(f: impl FnOnce(&mut Scheduler) -> T) -> T {
    Arch::without_interrupts(|| {
        let mut scheduler = SCHEDULER.lock();
        let Some(scheduler) = scheduler.as_mut() else {
            panic!("thread::init didn't run");
        };
        f(scheduler)
    })
}

/// Makes the running code the boot thread, and makes the idle thread.
/// `kernel::init` calls this once, after `heap::init` and before the timer
/// starts.
pub fn init() {
    let stack = paging::map_stack(STACK_SIZE);
    let idle = Box::new(Thread::new(
        Some(Arch::new_context(stack.clone(), idle)),
        Some(stack),
        None,
        None,
    ));
    let boot = Box::new(Thread::new(None, None, None, None));
    let scheduler = Scheduler {
        running: Some(boot),
        ready: VecDeque::with_capacity(1),
        blocked: Vec::with_capacity(1),
        idle: Some(idle),
        dead: Vec::with_capacity(1),
        leaving: None,
        threads: 1,
    };
    Arch::without_interrupts(|| *SCHEDULER.lock() = Some(scheduler));
    log::info!(
        "threads: boot and idle, {} KiB stacks with guard pages",
        STACK_SIZE >> 10
    );
}

/// Starts a thread that runs `run`, then exits. Panics if kernel space or
/// the frames run out.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "only the tests run kernel threads besides boot and idle so far"
    )
)]
pub fn spawn(run: impl FnOnce() + Send + 'static) {
    spawn_in(None, run);
}

/// Starts a thread that runs user code at `ip` in `space`, with stack
/// pointer `sp` and `arg` as its first argument. It ends through
/// `user::trap`, and its space is destroyed along with it. Panics if
/// kernel space or the frames run out.
pub fn spawn_user(space: paging::AddressSpace, ip: u64, sp: u64, arg: u64) {
    spawn_in(Some(space), move || Arch::enter_user(ip, sp, arg));
}

fn spawn_in(space: Option<paging::AddressSpace>, run: impl FnOnce() + Send + 'static) {
    let stack = paging::map_stack(STACK_SIZE);
    let mut thread = Some(Box::new(Thread::new(
        Some(Arch::new_context(stack.clone(), start)),
        Some(stack),
        Some(Box::new(run)),
        space,
    )));
    loop {
        let short = with(|scheduler| {
            let need = scheduler.threads + 1;
            if scheduler.ready.capacity() < need
                || scheduler.blocked.capacity() < need
                || scheduler.dead.capacity() < need
            {
                return Some(need);
            }
            scheduler.threads = need;
            if let Some(thread) = thread.take() {
                scheduler.ready.push_back(thread);
            }
            None
        });
        match short {
            Some(need) => grow(need),
            None => return,
        }
    }
}

/// Gives `ready`, `blocked` and `dead` room for `need` threads each.
/// Another thread may grow them meanwhile, so the caller checks again.
fn grow(need: usize) {
    let mut ready = VecDeque::with_capacity(need);
    let mut blocked = Vec::with_capacity(need);
    let mut dead = Vec::with_capacity(need);
    with(|scheduler| {
        if scheduler.ready.capacity() < need {
            ready.append(&mut scheduler.ready);
            mem::swap(&mut scheduler.ready, &mut ready);
        }
        if scheduler.blocked.capacity() < need {
            blocked.append(&mut scheduler.blocked);
            mem::swap(&mut scheduler.blocked, &mut blocked);
        }
        if scheduler.dead.capacity() < need {
            dead.append(&mut scheduler.dead);
            mem::swap(&mut scheduler.dead, &mut dead);
        }
    });
    // Whichever of each pair the scheduler doesn't keep is empty, and is
    // freed here with interrupts on.
}

/// The running thread's id.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing outside the tests blocks or waits for another thread yet"
    )
)]
pub fn id() -> u64 {
    with(|scheduler| running(scheduler).id)
}

fn running(scheduler: &mut Scheduler) -> &mut Thread {
    let Some(running) = scheduler.running.as_mut() else {
        panic!("no thread is running");
    };
    running
}

/// Blocks the running thread until [`unpark`] names it, or returns at once
/// if an unpark named it since its last park. May return without either,
/// so callers check what they wait for again.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing outside the tests blocks or waits for another thread yet"
    )
)]
pub fn park() -> Wake {
    park_with(None)
}

/// [`park`], but also returns once [`timer::ticks`] reaches `deadline`.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing outside the tests blocks or waits for another thread yet"
    )
)]
pub fn park_until(deadline: u64) -> Wake {
    park_with(Some(deadline))
}

fn park_with(deadline: Option<u64>) -> Wake {
    Arch::without_interrupts(|| {
        with(|scheduler| {
            assert!(scheduler.idle.is_some(), "the idle thread parked");
            let thread = running(scheduler);
            if thread.wake.is_none() && deadline.is_some_and(|deadline| deadline <= timer::ticks())
            {
                thread.wake = Some(Wake::TimedOut);
            }
            thread.deadline = deadline;
        });
        switch(Place::Blocked);
        with(|scheduler| {
            let thread = running(scheduler);
            thread.deadline = None;
            let Some(wake) = thread.wake.take() else {
                panic!("a parked thread resumed with nothing to wake it");
            };
            wake
        })
    })
}

/// Blocks the running thread for at least `ticks` ticks.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing outside the tests blocks or waits for another thread yet"
    )
)]
pub fn sleep(ticks: u64) {
    let deadline = timer::ticks() + ticks;
    while timer::ticks() < deadline {
        park_until(deadline);
    }
}

/// Readies the thread `id` names if it is parked, or makes its next park
/// return at once if it isn't. Does nothing if no live thread has that id.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing outside the tests blocks or waits for another thread yet"
    )
)]
pub fn unpark(id: u64) {
    with(|scheduler| {
        if let Ok(i) = scheduler
            .blocked
            .binary_search_by_key(&id, |thread| thread.id)
        {
            let mut thread = scheduler.blocked.remove(i);
            thread.wake = Some(Wake::Unparked);
            scheduler.ready.push_back(thread);
            return;
        }
        let thread = scheduler
            .running
            .iter_mut()
            .chain(scheduler.ready.iter_mut())
            .find(|thread| thread.id == id);
        if let Some(thread) = thread {
            thread.wake = Some(Wake::Unparked);
        }
    });
}

/// Lets the next ready thread run, if there is one.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing outside the tests blocks or waits for another thread yet"
    )
)]
pub fn yield_now() {
    Arch::without_interrupts(|| switch(Place::Ready));
}

/// Hands the CPU to the next ready thread. The kernel's trap handler calls
/// this on every timer tick, with interrupts off. Does nothing before
/// `init`.
pub fn tick() {
    if SCHEDULER.lock().is_none() {
        return;
    }
    with(|scheduler| {
        let now = timer::ticks();
        let mut i = 0;
        while i < scheduler.blocked.len() {
            if scheduler.blocked[i]
                .deadline
                .is_some_and(|deadline| deadline <= now)
            {
                let mut thread = scheduler.blocked.remove(i);
                thread.wake = Some(Wake::TimedOut);
                scheduler.ready.push_back(thread);
            } else {
                i += 1;
            }
        }
    });
    switch(Place::Ready);
}

/// Ends the running thread. First frees every thread that exited before
/// it, since nothing can free a thread's stack while the thread runs on
/// it.
pub fn exit() -> ! {
    reap();
    Arch::without_interrupts(|| switch(Place::Dead));
    panic!("a thread ran after it exited");
}

/// Ends the running thread from a trap handler, with interrupts still off,
/// and frees no other thread: the trap may have come in on a stack the CPU
/// starts the next such trap on, so nothing may preempt the thread there.
/// The idle thread, or the next thread to `exit`, frees it.
pub fn exit_from_trap() -> ! {
    switch(Place::Dead);
    panic!("a thread ran after it exited");
}

/// Switches from the running thread, which goes to `place`, or back to its
/// slot if it is the idle thread, to the next ready thread, or to the idle
/// thread if none is ready and the running one is blocked or dead. Returns
/// at once if none is ready and the running one isn't blocked or dead, or
/// if it would block with a wake already set. Interrupts are off.
fn switch(place: Place) {
    let to = with(|scheduler| {
        let place = if scheduler.idle.is_none() {
            assert!(!matches!(place, Place::Dead), "the idle thread exited");
            Place::Idle
        } else {
            place
        };
        if matches!(place, Place::Blocked) && running(scheduler).wake.is_some() {
            return None;
        }
        let mut next = match (scheduler.ready.pop_front(), place) {
            (Some(next), _) => next,
            (None, Place::Blocked | Place::Dead) => scheduler.idle.take()?,
            (None, Place::Ready | Place::Idle) => return None,
        };
        let Some(to) = next.context.take() else {
            panic!("a thread that isn't running has no context");
        };
        // The timer's handler runs this too, so the kernel's tables come
        // through the HAL call that takes no lock.
        match &next.space {
            Some(space) => space.activate(),
            None => Arch::activate_kernel_tables(),
        }
        let Some(leaving) = scheduler.running.replace(next) else {
            panic!("no thread was running");
        };
        scheduler.leaving = Some((leaving, place));
        Some(to)
    });
    if let Some(to) = to {
        arrived(Arch::switch(to));
    }
}

/// Puts the thread that switched to this one in its place, with `from`, the
/// context it left. Every thread runs this first when it resumes.
fn arrived(from: Context) {
    with(|scheduler| {
        let Some((mut thread, place)) = scheduler.leaving.take() else {
            panic!("a thread resumed with none leaving");
        };
        thread.context = Some(from);
        match place {
            Place::Ready => scheduler.ready.push_back(thread),
            Place::Blocked => {
                let (Ok(i) | Err(i)) = scheduler
                    .blocked
                    .binary_search_by_key(&thread.id, |blocked| blocked.id);
                scheduler.blocked.insert(i, thread);
            }
            Place::Dead => scheduler.dead.push(thread),
            Place::Idle => scheduler.idle = Some(thread),
        }
    });
}

/// Frees every thread that exited: unmaps its stack and frees its frames,
/// destroys its address space, then drops it. Interrupts are on, since the
/// page tables', the frames' and the heap's locks may be held by a thread a
/// tick preempted.
fn reap() {
    while let Some(mut thread) = with(|scheduler| {
        let thread = scheduler.dead.pop()?;
        scheduler.threads -= 1;
        Some(thread)
    }) {
        for page in thread
            .stack
            .iter()
            .flat_map(|stack| stack.clone().step_by(4096))
        {
            let Some(frame) = paging::unmap(page) else {
                panic!("stack page {page:#x} wasn't mapped");
            };
            frames::free(frame);
        }
        if let Some(space) = thread.space.take() {
            space.destroy(&mut frames::free, &mut frames::free);
        }
    }
}

/// A spawned thread's entry.
fn start(from: Context) -> ! {
    arrived(from);
    let run = with(|scheduler| scheduler.running.as_mut()?.run.take());
    Arch::enable_interrupts();
    if let Some(run) = run {
        run();
    }
    exit()
}

/// The idle thread's entry. It frees the threads that exited, then sleeps
/// until the next interrupt, whose tick switches to a ready thread if
/// there is one.
fn idle(from: Context) -> ! {
    arrived(from);
    Arch::enable_interrupts();
    loop {
        reap();
        Arch::wait_for_interrupt();
    }
}

/// The threads not freed yet, the boot thread included, and how many of
/// them are dead.
#[cfg(test)]
fn counts() -> (usize, usize) {
    with(|scheduler| (scheduler.threads, scheduler.dead.len()))
}

/// Whether the thread `id` names is parked.
#[cfg(test)]
fn is_blocked(id: u64) -> bool {
    with(|scheduler| {
        scheduler
            .blocked
            .binary_search_by_key(&id, |thread| thread.id)
            .is_ok()
    })
}

/// Waits until every thread but the boot thread is dead, then frees them,
/// so that the next test starts with the boot thread alone.
#[cfg(test)]
pub fn wait_for_exits() {
    crate::testing::spin_until("the threads to exit", || {
        let (threads, dead) = counts();
        threads - 1 == dead
    });
    reap();
    assert_eq!(counts(), (1, 0));
}

#[cfg(test)]
mod tests {
    use core::hint::spin_loop;
    use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

    use thisos_hal::{BacktraceEnd, Probe, Trap};

    use super::*;
    use crate::testing::{self, spin_until};

    // Neither the spawned threads nor this one yields, so all three make
    // progress only if ticks preempt them.
    #[test_case]
    fn ticks_preempt_threads_that_never_yield() {
        static COUNTS: [AtomicU64; 2] = [const { AtomicU64::new(0) }; 2];
        static STOP: AtomicBool = AtomicBool::new(false);
        for count in &COUNTS {
            spawn(move || {
                while !STOP.load(Ordering::Relaxed) {
                    count.fetch_add(1, Ordering::Relaxed);
                    spin_loop();
                }
            });
        }
        spin_until("both threads to run", || {
            COUNTS.iter().all(|count| count.load(Ordering::Relaxed) > 0)
        });
        STOP.store(true, Ordering::Relaxed);
        wait_for_exits();
    }

    // The first thread is dead before the second starts, so the second's
    // exit has to free it.
    #[test_case]
    fn exit_frees_the_threads_that_died_before_it() {
        static STACKS: [AtomicU64; 2] = [const { AtomicU64::new(0) }; 2];
        let run = || {
            for stack in &STACKS {
                spawn(|| stack.store(Arch::guard_page() + 4096, Ordering::Relaxed));
                spin_until("the thread to exit", || counts() == (2, 1));
            }
            let first = STACKS[0].load(Ordering::Relaxed);
            assert_eq!(paging::translate(first), None, "first stack at {first:#x}");
            let second = STACKS[1].load(Ordering::Relaxed);
            assert!(
                paging::translate(second).is_some(),
                "second stack at {second:#x}"
            );
            let free = frames::free_frames();
            reap();
            assert_eq!(counts(), (1, 0));
            assert_eq!(frames::free_frames() - free, STACK_SIZE / 4096);
            assert_eq!(
                paging::translate(second),
                None,
                "second stack at {second:#x}"
            );
        };
        // The first run grows the queues to fit, which they keep.
        run();
        let allocated = Arch::heap_stats().allocated;
        run();
        assert_eq!(Arch::heap_stats().allocated, allocated);
    }

    // Interrupts stay off in this thread, so no tick lets the spawned one
    // run in its place.
    #[test_case]
    fn yield_runs_the_next_ready_thread() {
        static RAN: AtomicBool = AtomicBool::new(false);
        spawn(|| RAN.store(true, Ordering::Relaxed));
        Arch::without_interrupts(|| {
            spin_until("the thread to run", || {
                yield_now();
                RAN.load(Ordering::Relaxed)
            });
        });
        wait_for_exits();
    }

    #[test_case]
    fn a_threads_stack_is_guarded_and_its_backtrace_ends_at_its_entry() {
        static DONE: AtomicBool = AtomicBool::new(false);
        static GUARD: AtomicU64 = AtomicU64::new(0);
        static FRAMES: AtomicUsize = AtomicUsize::new(0);
        static END: spin::Mutex<Option<BacktraceEnd>> = spin::Mutex::new(None);
        spawn(|| {
            GUARD.store(Arch::guard_page(), Ordering::Relaxed);
            let mut frames = 0;
            let end = Arch::backtrace(|_| frames += 1);
            FRAMES.store(frames, Ordering::Relaxed);
            *END.lock() = Some(end);
            DONE.store(true, Ordering::Relaxed);
        });
        spin_until("the thread to finish", || DONE.load(Ordering::Relaxed));
        let guard = GUARD.load(Ordering::Relaxed);
        assert_eq!(paging::translate(guard), None, "guard page {guard:#x}");
        assert_eq!(*END.lock(), Some(BacktraceEnd::Done));
        let frames = FRAMES.load(Ordering::Relaxed);
        assert!(frames >= 2, "{frames} frames");
        wait_for_exits();
    }

    // Unparks the middle one of three parked threads first, so the blocked
    // set has to find it between the others and keep them parked.
    #[test_case]
    fn unpark_wakes_only_the_thread_it_names() {
        static IDS: [AtomicU64; 3] = [const { AtomicU64::new(u64::MAX) }; 3];
        static WOKEN: [AtomicBool; 3] = [const { AtomicBool::new(false) }; 3];
        let run = || {
            for (id, woken) in IDS.iter().zip(&WOKEN) {
                woken.store(false, Ordering::Relaxed);
                spawn(move || {
                    id.store(super::id(), Ordering::Relaxed);
                    assert_eq!(park(), Wake::Unparked);
                    woken.store(true, Ordering::Relaxed);
                });
            }
            spin_until("all three threads to park", || {
                IDS.iter().all(|id| is_blocked(id.load(Ordering::Relaxed)))
            });
            unpark(IDS[1].load(Ordering::Relaxed));
            spin_until("the middle thread to wake", || {
                WOKEN[1].load(Ordering::Relaxed)
            });
            assert!(!WOKEN[0].load(Ordering::Relaxed));
            assert!(!WOKEN[2].load(Ordering::Relaxed));
            for id in [&IDS[0], &IDS[2]] {
                unpark(id.load(Ordering::Relaxed));
            }
            wait_for_exits();
            assert!(WOKEN.iter().all(|woken| woken.load(Ordering::Relaxed)));
        };
        // The first run grows the queues to fit, which they keep.
        run();
        let allocated = Arch::heap_stats().allocated;
        run();
        assert_eq!(Arch::heap_stats().allocated, allocated);
    }

    // The thread is ready, not parked, when the unpark comes, which
    // interrupts being off in this thread guarantees.
    #[test_case]
    fn an_unpark_before_the_park_isnt_lost() {
        static WAKE: spin::Mutex<Option<Wake>> = spin::Mutex::new(None);
        Arch::without_interrupts(|| {
            spawn(|| *WAKE.lock() = Some(park()));
            let Some(id) = with(|scheduler| scheduler.ready.back().map(|thread| thread.id)) else {
                panic!("the spawned thread isn't ready");
            };
            unpark(id);
        });
        spin_until("the thread to return from its park", || {
            WAKE.lock().is_some()
        });
        assert_eq!(*WAKE.lock(), Some(Wake::Unparked));
        wait_for_exits();
    }

    // The boot thread parks here, so the idle thread runs until the tick
    // that wakes it.
    #[test_case]
    fn park_until_and_sleep_wait_for_their_deadline() {
        let deadline = timer::ticks() + 5;
        assert_eq!(park_until(deadline), Wake::TimedOut);
        assert!(timer::ticks() >= deadline);
        assert_eq!(park_until(0), Wake::TimedOut);
        let start = timer::ticks();
        sleep(10);
        assert!(timer::ticks() >= start + 10);
    }

    // Shows the double fault's stack switch works from a thread's stack as
    // from the boot stack, and that the thread carries on afterwards.
    #[test_case]
    fn a_threads_stack_overflow_double_faults() {
        static DONE: AtomicBool = AtomicBool::new(false);
        spawn(|| {
            testing::expect_trap(|trap| trap == Trap::DoubleFault, Probe::OverflowStack);
            DONE.store(true, Ordering::Relaxed);
        });
        spin_until("the thread's overflow", || DONE.load(Ordering::Relaxed));
        wait_for_exits();
    }
}
