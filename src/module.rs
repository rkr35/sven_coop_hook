use crate::wide_format;

use std::ffi::CString;
use std::mem::{self, MaybeUninit};
use std::os::raw::c_char;
use std::ptr;

use thiserror::Error;
use winapi::shared::minwindef::HMODULE;
use winapi::um::{
    libloaderapi::{GetModuleHandleW, GetProcAddress},
    processthreadsapi::GetCurrentProcess,
    psapi::{GetModuleInformation, MODULEINFO},
};

#[derive(Error, Debug)]
pub enum ErrorKind<'a> {
    #[error("failed to get a handle to the module")]
    NullModule,

    #[error("failed to query module information")]
    GetModuleInformation,

    #[error("failed to find the address of CreateInterface")]
    GetCreateInterface,

    #[error("failed to convert the Rust string \"{0}\" to a C string because it contains an interior null byte at index {1}")]
    StrConversion(&'a str, usize),

    #[error("CreateInterface returned a null pointer for the interface \"{0}\"")]
    NullInterface(&'a str),
}

#[derive(Error, Debug)]
#[error("\"{module}\" error: {kind}")]
pub struct Error<'a> {
    module: String,
    kind: ErrorKind<'a>,
}

impl<'a> Error<'a> {
    fn new<'e>(module: &str, kind: ErrorKind<'e>) -> Error<'e> {
        Error {
            module: String::from(module),
            kind,
        }
    }
}

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub base: usize,
    pub size: usize,
    pub end: usize,
    create_interface: usize,
}

impl Module {
    pub fn from(name: &str) -> Result<Module, Error> {
        let (info, create_interface) = unsafe {
            let module = Module::get_handle(name)?;
            let info = Module::get_info(module).ok_or_else(|| Error::new(name, ErrorKind::GetModuleInformation))?;

            let create_interface = GetProcAddress(module, b"CreateInterface\0".as_ptr().cast());
    
            if create_interface.is_null() {
                return Err(Error::new(name, ErrorKind::GetCreateInterface));
            }

            (info, create_interface as usize)
        };
        
        let base = info.lpBaseOfDll as usize;
        let size = info.SizeOfImage as usize;

        let module = Module {
            name: String::from(name),
            base,
            size,
            end: base + size,
            create_interface,
        };

        log::info!("{:#x?}", module);

        Ok(module)
    }

    fn get_handle(name: &str) -> Result<HMODULE, Error> {
        let handle = unsafe {
            let wide_name = wide_format!("{}", name);
            GetModuleHandleW(wide_name.as_ptr())
        };

        if handle.is_null() {
            Err(Error::new(name, ErrorKind::NullModule))
        } else {
            Ok(handle)
        }
    }

    unsafe fn get_info(handle: HMODULE) -> Option<MODULEINFO> {
        let mut info = MaybeUninit::<MODULEINFO>::uninit();
        let size = mem::size_of::<MODULEINFO>() as u32;

        if GetModuleInformation(GetCurrentProcess(), handle, info.as_mut_ptr(), size) == 0 {
            None
        } else {
            Some(info.assume_init())
        }
    }

    pub fn create_interface<'n, T>(&self, name: &'n str) -> Result<&'static mut T, Error<'n>> {
        type CreateInterface<T> =
            extern "C" fn(name: *const c_char, return_code: *mut i32) -> Option<&'static mut T>;

        let create_interface =
            unsafe { mem::transmute::<usize, CreateInterface<T>>(self.create_interface) };
        
        let interface = 
            CString::new(name).map_err(|nul_error| {
                Error::new(
                    &self.name,
                    ErrorKind::StrConversion(name, nul_error.nul_position()),
                )
            })?;

        let interface =
            create_interface(interface.as_ptr(), ptr::null_mut());
            
        interface.ok_or_else(|| Error::new(&self.name, ErrorKind::NullInterface(name)))
    }
}
