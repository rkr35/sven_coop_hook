use crate::wide_format;
use std::mem::{self, MaybeUninit};

use log::info;
use winapi::um::{
    libloaderapi::GetModuleHandleW,
    processthreadsapi::GetCurrentProcess,
    psapi::{
        GetModuleInformation,
        MODULEINFO,
    }
};

#[derive(Debug)]
pub enum Error<'a> {
    NullModule(&'a str),
    GetModuleInformationFailed(&'a str),
}

#[derive(Debug)]
pub struct Module {
    pub base: usize,
    pub size: usize,
    pub end: usize,
}

impl Module {
    pub fn from(name: &str) -> Result<Self, Error> {
        let info = unsafe {
            let module = GetModuleHandleW(wide_format!("{}", name).as_ptr());

            if module.is_null() {
                return Err(Error::NullModule(name));
            }

            let mut info = MaybeUninit::<MODULEINFO>::uninit();
            let size = mem::size_of::<MODULEINFO>() as u32;

            if GetModuleInformation(GetCurrentProcess(), module, info.as_mut_ptr(), size) == 0 {
                return Err(Error::GetModuleInformationFailed(name));
            }
            
            info.assume_init()
        };
        
        let base = info.lpBaseOfDll as usize;
        let size = info.SizeOfImage as usize;

        let module = Self {
            base,
            size,
            end: base + size,
        };

        info!("{}: {:#x?}", name, module);
        
        Ok(module)
    }
}