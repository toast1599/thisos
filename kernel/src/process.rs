//! Processes: the handle table of the objects a process can reach, and how
//! the process ended. Its threads hold a process, and so do the handles
//! that name it. Ending a process closes its handles, which also breaks the
//! cycle that a handle to itself makes.

use alloc::sync::Arc;
use core::mem;

use spin::Mutex;
use thisos_abi::{Error, Rights};
use thisos_hal::{Cpu, Trap};
use thisos_handles::Table;

use crate::Arch;
use crate::memory::MemoryObject;

/// A kernel object, as a handle names it.
#[derive(Clone)]
pub enum Object {
    Process(
        #[expect(
            dead_code,
            reason = "no syscall acts on a process through its handle yet"
        )]
        Arc<Process>,
    ),
    Memory(Arc<MemoryObject>),
}

/// How a process ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    /// Through the exit syscall, with this status.
    Exited(u64),
    /// By a trap its code raised at `ip`.
    Killed { trap: Trap, ip: u64 },
}

pub struct Process {
    /// Taken with interrupts on, and never by a trap handler, since making
    /// a handle can allocate.
    handles: Mutex<Table<Object>>,
    /// Taken only with interrupts off, since the trap handler that kills a
    /// thread sets it.
    end: Mutex<Option<End>>,
}

impl Process {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            handles: Mutex::new(Table::new()),
            end: Mutex::new(None),
        })
    }

    /// Puts a handle to this process with `rights` in its own table, and
    /// returns it.
    pub fn insert_self(self: &Arc<Self>, rights: Rights) -> Result<u32, Error> {
        let object = Object::Process(Arc::clone(self));
        self.handles.lock().insert(object, rights)
    }

    /// Puts `object` in the table with `rights`, and returns its handle.
    pub fn insert(&self, object: Object, rights: Rights) -> Result<u32, Error> {
        self.handles.lock().insert(object, rights)
    }

    /// The object `handle` names, and the handle's rights.
    pub fn get(&self, handle: u32) -> Result<(Object, Rights), Error> {
        let handles = self.handles.lock();
        let (object, rights) = handles.get(handle)?;
        Ok((object.clone(), rights))
    }

    pub fn close(&self, handle: u32) -> Result<(), Error> {
        // The object drops after the lock is released.
        let _closed = self.handles.lock().remove(handle)?;
        Ok(())
    }

    pub fn duplicate(&self, handle: u32, rights: Rights) -> Result<u32, Error> {
        self.handles.lock().duplicate(handle, rights)
    }

    pub fn restrict(&self, handle: u32, rights: Rights) -> Result<(), Error> {
        self.handles.lock().restrict(handle, rights)
    }

    /// Records how the process ended, unless it ended already.
    pub fn set_end(&self, end: End) {
        Arch::without_interrupts(|| {
            self.end.lock().get_or_insert(end);
        });
    }

    #[cfg(test)]
    pub fn end(&self) -> Option<End> {
        Arch::without_interrupts(|| *self.end.lock())
    }

    /// Closes every handle in the table. The scheduler calls this when it
    /// frees the process's last thread, with interrupts on.
    pub fn close_handles(&self) {
        let _closed = mem::take(&mut *self.handles.lock());
    }
}
