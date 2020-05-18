use crate::game::pfnUserMsgHook;
use crate::yank::Yank;

use std::os::raw::{c_char, c_void};

use log::info;
use thiserror::Error;

// BEGIN MUTABLE GLOBAL STATE
use crate::hook::USER_MSG;
static mut HEALTH: pfnUserMsgHook = None;
// END MUTABLE GLOBAL STATE

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("unable to find the user_msg_s for \"{0}\"")]
    MsgNotFound(&'a str),
}

pub struct Hook {
    _health: Single,
}

impl Hook {
    pub unsafe fn new() -> Result<Self, Error<'static>> {
        Ok(Self {
            _health: Single::new("Health", &mut HEALTH, Some(my_health))?,
        })
    }
}

struct Single {
    name: &'static str,
    original: &'static mut pfnUserMsgHook,
}

impl Single {
    unsafe fn new(name: &'static str, original: &'static mut pfnUserMsgHook, new: pfnUserMsgHook) -> Result<Single, Error<'static>> {
        *original = hook(name, new)?;

        Ok(Single {
            name,
            original,
        })
    }
}

impl Drop for Single {
    fn drop(&mut self) {
        unsafe {
            let _ = hook(self.name, *self.original);
        }
    }
}

unsafe fn hook(message_name: &str, hook: pfnUserMsgHook) -> Result<pfnUserMsgHook, Error> {
    let user_msg = (*USER_MSG).find(message_name).ok_or(Error::MsgNotFound(message_name))?;

    // TODO: Investigate.
    // We are setting `pfn` in our hook thread.
    // The game may be accessing `pfn` in its game thread.
    // What are the possible reprecussions of these two actions?
    // Setting a usize is an atomic operation, but what if the game does the following sequence:

    // A = load(pfn)
    // ...other operations...
    // B = load(pfn)
    // code that assumes A == B

    // Our hook's thread may run concurrent to the thread running "...other operations..." so that
    // B == our hooked function instead of the original function.
    // A != B, breaking a previous invariant.

    // We can't really "inject" a mutex or synchronization primitive in the game.
    // SuspendThread + ResumeThread? 
    let original = (*user_msg).pfn;
    (*user_msg).pfn = hook;

    info!("Found user_msg_s \"{}\" at {:?}. The original function is at {:#x}.", message_name, user_msg, original.unwrap() as usize);
    Ok(original)
}

unsafe extern "C" fn my_health(name: *const c_char, size: i32, buf: *mut c_void) -> i32 {
    let original = HEALTH.yank();
    original(name, size, buf)
}