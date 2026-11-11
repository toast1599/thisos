//! Traps user code raises: its syscalls, and the faults that end it.

use thisos_abi::Syscall;
use thisos_hal::{Cpu, Platform, Trap, TrapFrame as _};

use crate::{Arch, thread};

/// How a user thread ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum End {
    /// Through the exit syscall, with this status.
    Exited(u64),
    /// By a trap its code raised at `ip`.
    Killed { trap: Trap, ip: u64 },
}

/// How the last user thread to end ended, for tests to wait on.
#[cfg(test)]
static ENDED: spin::Mutex<Option<End>> = spin::Mutex::new(None);

/// Handles a trap that [`TrapFrame::raised_by_user`](thisos_hal::TrapFrame::raised_by_user)
/// says user code raised: runs a syscall, and ends the thread on any other
/// trap, or on the exit syscall.
pub fn trap(trap: Trap, frame: &mut <Arch as Platform>::TrapFrame) {
    let end = if trap == Trap::Syscall {
        let (number, args) = frame.syscall();
        match Syscall::try_from(number) {
            Ok(Syscall::Exit) => End::Exited(args[0]),
            Err(e) => return frame.set_syscall_result(e.code(), 0),
        }
    } else {
        End::Killed {
            trap,
            ip: frame.ip(),
        }
    };
    match end {
        End::Exited(status) => log::info!("user thread exited with status {status:#x}"),
        End::Killed { trap, ip } => log::warn!("user thread killed by {trap} at {ip:#x}"),
    }
    // User code holds no kernel lock, so nothing this handler interrupted
    // does either, and exit frees threads with interrupts on.
    Arch::enable_interrupts();
    #[cfg(test)]
    {
        *ENDED.lock() = Some(end);
    }
    thread::exit()
}

#[cfg(test)]
mod tests {
    use core::ptr;
    use core::sync::atomic::AtomicU64;

    use thisos_abi::Error;
    use thisos_hal::{Access, AddressSpace as _, Protection, UserProgram};

    use super::*;
    use crate::frames;
    use crate::testing::spin_until;

    /// Runs `program` in a new space, with its code in the first user page
    /// and a stack page after it, and `arg` as its argument. Returns how it
    /// ended and where its code was, once its thread and space are freed,
    /// and checks that every frame they took came back: those of the space
    /// and its pages, and those of the thread's stack. Kernel-space tables
    /// the stack needed stay, so those are left out. Meanwhile it checks
    /// that this thread, a kernel thread, runs on the kernel's tables every
    /// time ticks switch back to it.
    fn run(program: UserProgram, arg: impl FnOnce(u64) -> u64) -> (End, u64) {
        let before = frames::free_frames();
        let Some(mut space) = crate::paging::new_space() else {
            panic!("no free frame for a space");
        };
        let code = space.user_space().start;
        let stack = code + 4096;
        let frame = page_frame();
        Arch::write_user_program(&frame, program);
        for (page, frame, protection) in [
            (code, frame, Protection::ReadExecute),
            (stack, page_frame(), Protection::ReadWrite),
        ] {
            if let Err(e) = space.map(page, frame, protection, &mut frames::alloc) {
                panic!("mapping {page:#x} failed: {:?}", e.kind);
            }
        }
        let space_frames = before - frames::free_frames();
        *ENDED.lock() = None;
        thread::spawn_user(space, code, stack + 4096, arg(stack));
        // User code allocates nothing, and nothing frees the thread until
        // wait_for_exits, so this counts its stack whenever it runs.
        let spawned = frames::free_frames();
        spin_until("the user thread to end", || {
            assert_eq!(Arch::translate_active(code), None, "user code's page");
            ENDED.lock().is_some()
        });
        thread::wait_for_exits();
        assert_eq!(
            frames::free_frames(),
            spawned + space_frames + thread::STACK_SIZE / 4096,
            "frames leaked"
        );
        let Some(end) = *ENDED.lock() else {
            unreachable!("ENDED was set");
        };
        (end, code)
    }

    fn page_frame() -> thisos_frames::Frame {
        let Some(frame) = frames::alloc() else {
            panic!("no free frame");
        };
        frame
    }

    #[test_case]
    fn a_syscall_keeps_registers_and_an_unknown_one_fails() {
        let (end, _) = run(UserProgram::Syscalls, |_| 0);
        assert_eq!(end, End::Exited(Error::UnknownSyscall.code()));
    }

    #[test_case]
    fn user_code_reads_its_own_stack() {
        let (end, _) = run(UserProgram::Read, |stack| stack);
        assert_eq!(end, End::Exited(0));
    }

    #[test_case]
    fn reading_kernel_memory_kills_the_thread() {
        static TARGET: AtomicU64 = AtomicU64::new(0);
        let Ok(address) = u64::try_from(ptr::from_ref(&TARGET).addr()) else {
            panic!("a static's address doesn't fit a u64");
        };
        let (end, code) = run(UserProgram::Read, |_| address);
        let trap = Trap::PageFault {
            address,
            access: Access::Read,
        };
        assert_eq!(end, End::Killed { trap, ip: code });
    }

    #[test_case]
    fn a_privileged_instruction_kills_the_thread() {
        let (end, code) = run(UserProgram::Privileged, |_| 0);
        assert!(
            matches!(end, End::Killed { trap: Trap::Other { .. }, ip } if ip == code),
            "{end:?}"
        );
    }

    // The kernel saves no SIMD state, so user code may not use it.
    #[test_case]
    fn simd_kills_the_thread() {
        let (end, code) = run(UserProgram::Simd, |_| 0);
        assert!(
            matches!(end, End::Killed { trap: Trap::Other { .. }, ip } if ip == code),
            "{end:?}"
        );
    }

    // The user thread spins through many ticks, each of which preempts it
    // in ring 3 and switches to this thread.
    #[test_case]
    fn ticks_preempt_user_code() {
        let (end, _) = run(UserProgram::Spin, |_| 0);
        assert_eq!(end, End::Exited(0));
    }
}
