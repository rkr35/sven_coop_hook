use log::info;
use thiserror::Error;

// BEGIN MUTABLE GLOBAL STATE
// END MUTABLE GLOBAL STATE

#[derive(Error, Debug)]
pub enum Error<'a> {
}

pub struct Hook {
}

impl Hook {
    pub unsafe fn new() -> Result<Self, Error<'static>> {
        Ok(Self {
            
        })
    }
}