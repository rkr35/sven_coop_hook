use crate::game::pfnUserMsgHook;

use std::os::raw::c_void;

use log::info;
use thiserror::Error;

// BEGIN MUTABLE GLOBAL STATE
use crate::hook::USER_MSG;
// END MUTABLE GLOBAL STATE

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("unable to find the user_msg_s for \"{0}\"")]
    MsgNotFound(&'a str),
}

pub struct Hook {
}

impl Hook {
    pub unsafe fn new() -> Result<Self, Error<'static>> {
        Ok(Self {
        })
    }
}

struct Single {
    name: &'static str,
    original: &'static mut pfnUserMsgHook,
}

impl Single {
    unsafe fn _new(name: &'static str, original: &'static mut pfnUserMsgHook,
                  new: pfnUserMsgHook) -> Result<Single, Error<'static>> {

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

unsafe fn _print_buffer(size: i32, buf: *mut c_void) {
    let size = size as usize;
    let buf: *const u8 = buf.cast();
    let buf = std::slice::from_raw_parts(buf, size);
    info!("{:?}", buf);
}
