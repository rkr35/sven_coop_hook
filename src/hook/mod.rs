use crate::hw;
use crate::idle;
use crate::module::{self, Module};

use std::ptr;

use log::{error, info};
use thiserror::Error;

mod panel;

// BEGIN MUTABLE GLOBAL STATE
pub static mut SURFACE: *const hw::Surface = ptr::null();
// END MUTABLE GLOBAL STATE

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("{0}")]
    Module(module::Error<'a>),

    #[error("panel hook error: {0}")]
    Panel(panel::Error<'a>),
}

impl<'a> From<module::Error<'a>> for Error<'a> {
    fn from(e: module::Error) -> Error {
        Error::Module(e)
    }
}

impl<'a> From<panel::Error<'a>> for Error<'a> {
    fn from(e: panel::Error) -> Error {
        Error::Panel(e)
    }
}

struct Hook {
    _panel: panel::Hook,
}

impl Hook {
    fn new(modules: &Modules) -> Result<Hook, Error<'static>> {
        unsafe { SURFACE = modules.hw.create_interface::<hw::Surface>(hw::surface::INTERFACE)?; }
        info!("surface = {:#x?}", unsafe { SURFACE });

        let screen_fade = modules.hw.find_string("ScreenFade");
        info!("screen_fade = {:x?}", screen_fade);

        Ok(Hook {
            _panel: panel::Hook::new(&modules.vgui2)?
        })
    }
}

struct Modules {
    hw: Module,
    vgui2: Module,
}

impl Modules {
    fn new() -> Result<Modules, module::Error<'static>> {
        Ok(Modules {
            hw: Module::from("hw.dll")?,
            vgui2: Module::from("vgui2.dll")?,
        })
    }
}

pub fn run() -> Result<(), Error<'static>> {
    let modules = Modules::new()?;
    let _hook = Hook::new(&modules)?;
    idle();
    Ok(())
}
