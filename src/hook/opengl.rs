use crate::game::GLenum;
use crate::module::Module;
use crate::single_thread_verifier;

use std::ffi::c_void;
use std::mem;
use std::ptr;

use bstr::BStr;
use detours_sys::{DetourAttach, DetourDetach, DetourTransactionBegin, DetourTransactionCommit};
use log::info;
use thiserror::Error;

// BEGIN MUTABLE GLOBAL STATE
static mut ORIGINAL_GL_BEGIN: *mut c_void = ptr::null_mut();
// END MUTABLE GLOBAL STATE

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("Unable to find the address of {0}")]
    GetProcAddress(&'a BStr)
}

pub struct Hook {}

impl Hook {
    pub unsafe fn new(opengl: &Module) -> Result<Self, Error<'static>> {
        add_detours(opengl)?;
        Ok(Self {})
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        unsafe {
            remove_detours();
        }
    }
}

unsafe extern "system" fn my_gl_begin(mode: GLenum) {
    single_thread_verifier::assert();
    type GlBegin = unsafe extern "system" fn (mode: GLenum);
    let original = mem::transmute::<*mut c_void, GlBegin>(ORIGINAL_GL_BEGIN);
    original(mode);
}

unsafe fn add_detours(opengl: &Module) -> Result<(), Error<'static>> {
    const GL_BEGIN: [u8; 8] = *b"glBegin\0";

    ORIGINAL_GL_BEGIN = opengl.get_proc_address(&GL_BEGIN)
        .ok_or_else(|| Error::GetProcAddress(GL_BEGIN.as_ref().into()))? as *mut c_void;
    
    info!("ORIGINAL_GL_BEGIN={:?}", ORIGINAL_GL_BEGIN);
    // todo: Check for error codes and bubble up.
    DetourTransactionBegin();
    DetourAttach(&mut ORIGINAL_GL_BEGIN, my_gl_begin as *mut _);
    DetourTransactionCommit();

    Ok(())
}

unsafe fn remove_detours() {
    DetourTransactionBegin();
    DetourDetach(&mut ORIGINAL_GL_BEGIN, my_gl_begin as *mut _);
    DetourTransactionCommit();
}