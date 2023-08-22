use rlimit::Resource;
use std::{
    alloc::Layout,
    error::Error,
    fmt::{Debug, Display},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Mutex,
    },
};

const RESERVED_FOR_OOM: usize = 1024 * 1024 * 128;
static CURRENT_LIMIT: AtomicUsize = AtomicUsize::new(usize::MAX);
static OOM_TRIGGERED: AtomicBool = AtomicBool::new(false);
static LIMIT_MEMORY_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug)]
pub enum MemoryLimitError {
    MemoryLimitInc,
    OOMTriggered,
    IOError(std::io::Error),
}
impl Display for MemoryLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}
impl Error for MemoryLimitError {}

pub fn limit_memory(limit: usize) -> Result<(), MemoryLimitError> {
    let locked = LIMIT_MEMORY_LOCK.lock().unwrap();
    if OOM_TRIGGERED.load(Ordering::SeqCst) {
        return Err(MemoryLimitError::OOMTriggered);
    }
    if CURRENT_LIMIT.load(Ordering::SeqCst) < limit {
        return Err(MemoryLimitError::MemoryLimitInc);
    }
    CURRENT_LIMIT.store(limit, Ordering::SeqCst);
    let soft_limit = limit;
    let hard_limit = limit.checked_add(RESERVED_FOR_OOM).unwrap_or(limit);
    Resource::AS
        .set(soft_limit as u64, hard_limit as u64)
        .map_err(MemoryLimitError::IOError)?;
    if OOM_TRIGGERED.load(Ordering::SeqCst) {
        return Err(MemoryLimitError::OOMTriggered);
    }
    drop(locked);
    Ok(())
}

#[allow(unused)]
fn oom_handler(layout: Layout) {
    let locked = LIMIT_MEMORY_LOCK.lock();
    OOM_TRIGGERED.store(true, Ordering::SeqCst);
    let current_limit = CURRENT_LIMIT.load(Ordering::SeqCst);
    let hard_limit = current_limit
        .checked_add(RESERVED_FOR_OOM)
        .unwrap_or(current_limit);
    Resource::AS.set(hard_limit as u64, hard_limit as u64).ok();
    drop(locked);
    panic!("OOM: {:?}", layout);
}

#[cfg(nightly)]
#[ctor::ctor]
fn set_oom_handler() {
    std::alloc::set_alloc_error_hook(oom_handler);
}
