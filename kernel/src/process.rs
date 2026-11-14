//! Processes: the handle table of the objects a process can reach, and how
//! the process ended. Its threads hold a process, and so do the handles
//! that name it. Ending a process closes its handles, which also breaks the
//! cycle that a handle to itself makes.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem;

use spin::Mutex;
use thisos_abi::{Error, Rights};
use thisos_hal::{Cpu, Trap};
use thisos_handles::Table;

use crate::Arch;
use crate::channel::Endpoint;
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
    Channel(Arc<Endpoint>),
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

    /// Closes every handle in `handles` and returns their objects and
    /// rights, if each names an object `check` accepts; see
    /// [`Table::remove_all`]. `check` runs under the table's lock.
    pub fn take(
        &self,
        handles: &[u32],
        check: impl FnMut(&Object, Rights) -> Result<(), Error>,
    ) -> Result<Vec<(Object, Rights)>, Error> {
        self.handles.lock().remove_all(handles, check)
    }

    /// Puts every one of `objects` in the table, then calls `then` with
    /// their handles, in order, all under the table's lock. If there isn't
    /// room for them all, or `then` fails, it fails with that error and
    /// hands `objects` back, leaving the table as it was.
    pub fn insert_all<R>(
        &self,
        objects: Vec<(Object, Rights)>,
        then: impl FnOnce(&[u32]) -> Result<R, Error>,
    ) -> Result<R, (Error, Vec<(Object, Rights)>)> {
        let mut table = self.handles.lock();
        if table.room() < objects.len() {
            return Err((Error::NoHandles, objects));
        }
        // Both reserved first, so that undoing allocates nothing.
        let (mut handles, mut undone) = (Vec::new(), Vec::new());
        if handles.try_reserve_exact(objects.len()).is_err()
            || undone.try_reserve_exact(objects.len()).is_err()
        {
            return Err((Error::NoMemory, objects));
        }
        for (object, rights) in objects {
            let Ok(handle) = table.insert(object, rights) else {
                unreachable!("the table had room");
            };
            handles.push(handle);
        }
        then(&handles).map_err(|e| {
            for &handle in &handles {
                let Ok(entry) = table.remove(handle) else {
                    unreachable!("{handle:#x} was made under the same lock");
                };
                undone.push(entry);
            }
            (e, undone)
        })
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
