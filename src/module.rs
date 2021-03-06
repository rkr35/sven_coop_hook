use crate::memory;
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

    #[error("CreateInterface returned a bad pointer for the interface \"{0}\": {1}")]
    BadInterface(&'a str, memory::Error),
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
    module: HMODULE,
    pub name: String,
    pub base: usize,
    pub size: usize,
    pub end: usize,
}

impl Module {
    pub fn from(name: &str) -> Result<Module, Error> {
        let (module, info) = unsafe {
            let module = Module::get_handle(name)?;

            let info = Module::get_info(module).ok_or_else(|| Error::new(name, ErrorKind::GetModuleInformation))?;
            
            (module, info)
        };
        
        let base = info.lpBaseOfDll as usize;
        let size = info.SizeOfImage as usize;

        let module = Module {
            module,
            name: String::from(name),
            base,
            size,
            end: base + size,
        };
        
        Ok(module)
    }

    pub unsafe fn get_proc_address(&self, proc: &[u8]) -> Option<usize> {
        get_proc_address(self.module, proc)
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

        #[allow(clippy::cast_possible_truncation)]
        let size = mem::size_of::<MODULEINFO>() as u32;

        if GetModuleInformation(GetCurrentProcess(), handle, info.as_mut_ptr(), size) == 0 {
            None
        } else {
            Some(info.assume_init())
        }
    }

    pub fn find_bytes(&self, find_me: &[u8]) -> Option<*const u8> {
        let memory = unsafe {
            let base = self.base as *const u8;
            std::slice::from_raw_parts(base, self.size)
        };
    
        memory
            .windows(find_me.len())
            .find(|window| *window == find_me)
            .map(|window| window.as_ptr())
    }

    pub fn find_string(&self, string: &str) -> Option<*const u8> {
        self.find_bytes(string.as_bytes())
    }

    pub fn _find_pattern(&self, pattern: &[Option<u8>]) -> Option<usize> {
        let memory = unsafe {
            let base = self.base as *const u8;
            std::slice::from_raw_parts(base, self.size)
        };

        memory
            .windows(pattern.len())
            .find(|window|
                pattern
                    .iter()
                    .zip(window.iter())
                    .all(|(pattern_byte, module_byte)| pattern_byte.map_or(true, |p| p == *module_byte)))
            .map(|window| window.as_ptr() as usize)
    }
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct GameModule {
    pub module: Module,
    create_interface: usize,
}

impl GameModule {
    pub fn from(name: &str) -> Result<GameModule, Error> {
        let module = Module::from(name)?;

        let create_interface = unsafe {
            get_proc_address(module.module, b"CreateInterface\0")
                .ok_or_else(|| Error::new(name, ErrorKind::GetCreateInterface))?
        };

        Ok(GameModule {
            module,
            create_interface, 
        })
    }

    pub fn create_interface<'n, T>(&self, name: &'n str) -> Result<*mut T, Error<'n>> {
        type CreateInterface<T> =
            extern "C" fn(name: *const c_char, return_code: *mut i32) -> *mut T;

        let create_interface =
            unsafe { mem::transmute::<usize, CreateInterface<T>>(self.create_interface) };
        
        let interface = 
            CString::new(name).map_err(|nul_error| {
                Error::new(
                    &self.module.name,
                    ErrorKind::StrConversion(name, nul_error.nul_position()),
                )
            })?;

        let interface =
            create_interface(interface.as_ptr(), ptr::null_mut());
        
        memory::ptr_check(interface)
            .map_err(|e| Error::new(&self.module.name, ErrorKind::BadInterface(name, e)))?;

        Ok(interface)
    }
}

unsafe fn get_proc_address(module: HMODULE, proc: &[u8]) -> Option<usize> {
    let pointer = GetProcAddress(module, proc.as_ptr().cast());

    if pointer.is_null() {
        None
    } else {
        Some(pointer as usize)
    }
}