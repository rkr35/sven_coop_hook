use crate::hw;
use crate::idle;
use crate::module::{self, Module};

use std::mem;
use std::ptr;

use log::{error, info};
use thiserror::Error;

mod panel;

// BEGIN MUTABLE GLOBAL STATE
pub static mut SURFACE: *const hw::Surface = ptr::null();
pub static mut ENGINE_FUNCS: *const EngineFuncs = ptr::null();
pub static mut CLIENT_FUNCS: *const ClientFuncs = ptr::null();
// END MUTABLE GLOBAL STATE

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("{0}")]
    Module(module::Error<'a>),

    #[error("panel hook error: {0}")]
    Panel(panel::Error<'a>),

    #[error("could not find address of the string literal \"{0}\"")]
    NotFoundStringLit(&'a str),

    #[error("bytes not found for {0}")]
    NotFoundBytes(&'a str),
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

        init_engine_and_client_funcs(&modules.hw)?;

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

#[repr(usize)]
pub enum EngineFuncsTable {
    GetWindowCenterX = 33,
    NumEntries = 131,
}

#[repr(C)]
pub struct EngineFuncs {
    functions: [usize; EngineFuncsTable::NumEntries as usize]
}

impl EngineFuncs {
    pub fn get_window_center_x(&self) -> i32 {
        type GetWindowCenterX = extern "C" fn() -> i32;
        let address = self.functions[EngineFuncsTable::GetWindowCenterX as usize];
        let function: GetWindowCenterX = unsafe { mem::transmute(address) };
        function()
    }
}

#[repr(usize)]
pub enum ClientFuncsTable {
    NumEntries = 43,
}

#[repr(C)]
pub struct ClientFuncs {
    functions: [usize; ClientFuncsTable::NumEntries as usize]
}

fn get_screen_fade_instruction(hw: &Module) -> Result<*const u8, Error<'static>> {
    const SCREEN_FADE: &str = "ScreenFade";
    const PUSH: u8 = 0x68;

    let screen_fade = hw
        .find_string(SCREEN_FADE)
        .ok_or(Error::NotFoundStringLit(SCREEN_FADE))?;

    let mut push_screen_fade: [u8; 5] = [PUSH, 0, 0, 0, 0];
    (&mut push_screen_fade[1..])
        .copy_from_slice(&(screen_fade as usize).to_le_bytes());

    Ok(hw
        .find_bytes(&push_screen_fade)
        .ok_or(Error::NotFoundBytes("push ScreenFade instruction"))?)
}

fn init_engine_and_client_funcs(hw: &Module) -> Result<(), Error<'static>> {
    let screen_fade = get_screen_fade_instruction(hw)?;

    unsafe {
        let engine_funcs: *const *const EngineFuncs = screen_fade.add(13).cast();
        ENGINE_FUNCS = engine_funcs.read_unaligned();

        let client_funcs: *const *const ClientFuncs = screen_fade.add(19).cast();
        CLIENT_FUNCS = client_funcs.read_unaligned();

        info!("engine = {:?}, client = {:?}", ENGINE_FUNCS, CLIENT_FUNCS);
    }

    Ok(())
}

pub fn run() -> Result<(), Error<'static>> {
    let modules = Modules::new()?;
    let _hook = Hook::new(&modules)?;
    idle();
    Ok(())
}
