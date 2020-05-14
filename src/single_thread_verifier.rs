#[cfg(not(feature = "single_thread_verifier"))]
pub fn assert() {}

#[cfg(feature = "single_thread_verifier")]
pub fn assert() {
    use std::thread::{self, ThreadId};
    use once_cell::sync::Lazy;
    static THE_ONE_THREAD_ID: Lazy<ThreadId> = Lazy::new(|| thread::current().id()); 

    let current_thread_id = thread::current().id();

    if current_thread_id != *THE_ONE_THREAD_ID {
        log::error!(
            "Current thread ID ({:?}) does NOT equal THE_ONE_THREAD_ID ({:?}).",
            current_thread_id,
            *THE_ONE_THREAD_ID
        );
    }
}