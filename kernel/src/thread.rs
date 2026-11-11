//! Kernel threads and the round-robin scheduler. Every timer tick hands the
//! CPU to the next ready thread, and the idle thread runs only when no
//! thread is ready. The boot thread runs on the loader's stack; every other
//! thread has a stack from `paging::map_stack`. A user thread is a kernel
//! thread that owns an address space and has left for user code in it;
//! every switch loads the next thread's space, or the kernel's tables.
//!
//! The scheduler's lock is only ever held with interrupts off, so the
//! timer's handler never finds it held, and nothing allocates or frees
//! while holding it: a tick can preempt a thread that holds the heap's lock,
//! and the heap would then spin with interrupts off for good. `spawn`
//! reserves room in the queues for every thread, so moving one never
//! allocates.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing outside the tests spawns a thread until init runs in user mode"
    )
)]

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::mem;
use core::ops::Range;

use spin::Mutex;
use thisos_hal::{AddressSpace as _, Cpu, Platform};

use crate::{Arch, frames, paging};

type Context = <Arch as Platform>::Context;

/// The size of every thread's stack but the boot thread's. aarch64's trap
/// entries tell an overflow by bit 16 of the stack pointer, which needs
/// 64 KiB stacks aligned to 128 KiB, as `map_stack` aligns them.
pub const STACK_SIZE: usize = 64 << 10;

struct Thread {
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
}

/// Where the thread that switches away goes.
#[derive(Clone, Copy)]
enum Place {
    Ready,
    Dead,
    Idle,
}

struct Scheduler {
    running: Option<Box<Thread>>,
    ready: VecDeque<Box<Thread>>,
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
    /// thread not: the room `ready` and `dead` each need.
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
    let idle = Box::new(Thread {
        context: Some(Arch::new_context(stack.clone(), idle)),
        stack: Some(stack),
        run: None,
        space: None,
    });
    let boot = Box::new(Thread {
        context: None,
        stack: None,
        run: None,
        space: None,
    });
    let scheduler = Scheduler {
        running: Some(boot),
        ready: VecDeque::with_capacity(1),
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
    let mut thread = Some(Box::new(Thread {
        context: Some(Arch::new_context(stack.clone(), start)),
        stack: Some(stack),
        run: Some(Box::new(run)),
        space,
    }));
    loop {
        let short = with(|scheduler| {
            let need = scheduler.threads + 1;
            if scheduler.ready.capacity() < need || scheduler.dead.capacity() < need {
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

/// Gives `ready` and `dead` room for `need` threads each. Another thread
/// may grow them meanwhile, so the caller checks again.
fn grow(need: usize) {
    let mut ready = VecDeque::with_capacity(need);
    let mut dead = Vec::with_capacity(need);
    with(|scheduler| {
        if scheduler.ready.capacity() < need {
            ready.append(&mut scheduler.ready);
            mem::swap(&mut scheduler.ready, &mut ready);
        }
        if scheduler.dead.capacity() < need {
            dead.append(&mut scheduler.dead);
            mem::swap(&mut scheduler.dead, &mut dead);
        }
    });
    // Whichever of each pair the scheduler doesn't keep is empty, and is
    // freed here with interrupts on.
}

/// Lets the next ready thread run, if there is one.
pub fn yield_now() {
    Arch::without_interrupts(|| switch(Place::Ready));
}

/// Hands the CPU to the next ready thread. The kernel's trap handler calls
/// this on every timer tick, with interrupts off. Does nothing before
/// `init`.
pub fn tick() {
    if SCHEDULER.lock().is_some() {
        switch(Place::Ready);
    }
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
/// thread if none is ready and the running one is dead. Returns at once if
/// none is ready and the running one isn't dead. Interrupts are off.
fn switch(place: Place) {
    let to = with(|scheduler| {
        let place = if scheduler.idle.is_none() {
            assert!(!matches!(place, Place::Dead), "the idle thread exited");
            Place::Idle
        } else {
            place
        };
        let mut next = match (scheduler.ready.pop_front(), place) {
            (Some(next), _) => next,
            (None, Place::Dead) => scheduler.idle.take()?,
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
