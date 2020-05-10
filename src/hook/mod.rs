use crate::game::client::{ClientFuncs, PlayerMove};
use crate::game::engine::EngineFuncs;
use crate::game::hw;
use crate::idle;
use crate::memory;
use crate::module::{self, Module};

use std::ptr;

use log::{error, info};
use thiserror::Error;

mod client;
mod panel;

// BEGIN MUTABLE GLOBAL STATE
pub static mut SURFACE: *const hw::Surface = ptr::null();
pub static mut ENGINE_FUNCS: *const EngineFuncs = ptr::null();
pub static mut ORIGINAL_CLIENT_FUNCS: Option<ClientFuncs> = None;
pub static mut PLAYER_MOVE: *const PlayerMove = ptr::null();
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

    #[error("patch error: {0}")]
    Patch(#[from] memory::Error),
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
    _client: client::Hook,
    _panel: panel::Hook,
}

impl Hook {
    fn new(modules: &Modules) -> Result<Hook, Error<'static>> {
        let screen_fade = get_screen_fade_instruction(&modules.hw)?;

        let _client = unsafe {
            SURFACE = modules.hw.create_interface::<hw::Surface>(hw::surface::INTERFACE)?;
            info!("SURFACE = {:?}", SURFACE);

            let engine_funcs: *const *const EngineFuncs = screen_fade.add(13).cast();
            ENGINE_FUNCS = engine_funcs.read_unaligned();
            memory::ptr_check(ENGINE_FUNCS)?;
            info!("ENGINE_FUNCS = {:?}", ENGINE_FUNCS);
    
            let client_funcs: *const *mut ClientFuncs = screen_fade.add(19).cast();
            let client_funcs = client_funcs.read_unaligned();
            memory::ptr_check(client_funcs)?;
            info!("client_funcs = {:?}", client_funcs);
            ORIGINAL_CLIENT_FUNCS = (*client_funcs).clone().into();
    
            let player_move: *const *const PlayerMove = screen_fade.add(36).cast();
            PLAYER_MOVE = player_move.read_unaligned();
            memory::ptr_check(PLAYER_MOVE)?;
            info!("PLAYER_MOVE = {:?}", PLAYER_MOVE);
    
            client::Hook::new(client_funcs)
        };

        Ok(Hook {
            _client,
            _panel: panel::Hook::new(&modules.vgui2)?,
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

pub fn run() -> Result<(), Error<'static>> {
    let modules = Modules::new()?;
    let _hook = Hook::new(&modules)?;
    idle();
    Ok(())
}
