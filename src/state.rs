use std::sync::atomic::AtomicU64;

#[derive(Default)]
pub struct AppState {
    sleeptime_ms: AtomicU64,
}

impl AppState {
    
    pub fn new(sleeptime: u64) -> Self {
        Self {
            sleeptime_ms: AtomicU64::new(sleeptime),
        }
    }
    
    pub fn set_sleeptime(&self, sleeptime: u64) {
        self.sleeptime_ms.store(sleeptime, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn sleeptime(&self) -> u64 {
        self.sleeptime_ms.load(std::sync::atomic::Ordering::Relaxed)
    }
}
