use crate::wide_format;
use std::ffi::CString;
use std::mem::{self, MaybeUninit};
use std::os::raw::c_char;
use std::ptr;

use log::info;
use winapi::shared::minwindef::HMODULE;
use winapi::um::{
    libloaderapi::{GetModuleHandleW, GetProcAddress},
    processthreadsapi::GetCurrentProcess,
    psapi::{GetModuleInformation, MODULEINFO},
};

#[derive(Debug)]
pub enum ErrorKind {
    NullModule,
    GetModuleInformationFailed,
    GetCreateInterfaceFailed,
    RustStrToCStrErr,
    InterfaceIsNull,
}

#[derive(Debug)]
pub struct Error<'a> {
    module: &'a str,
    kind: ErrorKind,
}

impl<'a> Error<'a> {
    pub fn new(module: &str, kind: ErrorKind) -> Error {
        Error { module, kind }
    }
}

#[derive(Debug)]
pub struct Module<'a> {
    pub name: &'a str,
    pub base: usize,
    pub size: usize,
    pub end: usize,
    create_interface: usize,
}

impl<'a> Module<'a> {
    pub fn from(name: &str) -> Result<Module, Error> {
        use ErrorKind::*;

        let (info, create_interface) = unsafe {
            let module = Self::get_handle(name)?;
            let info = Self::get_info(module).ok_or(Error::new(name, GetModuleInformationFailed))?;

            let proc = CString::new("CreateInterface").map_err(|_| Error::new(name, RustStrToCStrErr))?;
            let create_interface = GetProcAddress(module, proc.as_ptr());
    
            if create_interface.is_null() {
                return Err(Error::new(name, GetCreateInterfaceFailed));
            }

            (info, create_interface as usize)
        };
        
        let base = info.lpBaseOfDll as usize;
        let size = info.SizeOfImage as usize;

        let module = Module {
            name,
            base,
            size,
            end: base + size,
            create_interface,
        };

        info!("{:#x?}", module);

        Ok(module)
    }

    fn get_handle(name: &str) -> Result<HMODULE, Error> {
        let handle = unsafe { GetModuleHandleW(wide_format!("{}", name).as_ptr()) };

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

    pub fn create_interface<T>(&self, name: &str) -> Result<&'static mut T, Error> {
        type CreateInterface<T> = extern fn(name: *const c_char, return_code: *mut i32) -> Option<&'static mut T>;
        
        let function = unsafe { mem::transmute::<usize, CreateInterface<T>>(self.create_interface) };

        let interface = CString::new(name).map_err(|_| Error::new(self.name, ErrorKind::RustStrToCStrErr))?;
        let interface = function(interface.as_ptr(), ptr::null_mut());

        interface.ok_or_else(|| Error::new(self.name, ErrorKind::InterfaceIsNull))
    }
}
