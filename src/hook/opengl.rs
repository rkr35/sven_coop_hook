use std::marker::PhantomData;

use log::info;
use thiserror::Error;

// BEGIN MUTABLE GLOBAL STATE
// END MUTABLE GLOBAL STATE

struct Original {

}

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("remove me")]
    phantom(PhantomData<&'a ()>),
}

pub struct Hook {
}

impl Hook {
    pub unsafe fn new() -> Result<Self, Error<'static>> {

        Ok(Self {
        })
    }
}