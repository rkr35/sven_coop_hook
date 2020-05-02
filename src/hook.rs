use crate::module::{Module, Error as ModuleError};

#[derive(Debug)]
pub enum Error<'a> {
    Module(ModuleError<'a>)
}

impl<'a> From<ModuleError<'a>> for Error<'a> {
    fn from(m: ModuleError) -> Error {
        Error::Module(m)
    }
}

pub struct Hook {
    _hw: Module,
}

impl Hook {
    pub fn new() -> Result<Self, Error<'static>> {
        Ok(Self {
            _hw: Module::from("hw.dll")?
        })
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        
    }
}