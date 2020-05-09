use crate::hw;
use crate::idle;
use crate::memory;
use crate::module::{self, Module};

use std::mem::{self, MaybeUninit};
use std::ptr;

use log::{error, info};
use thiserror::Error;
use ultraviolet::Vec3 as Vector;

mod panel;

// BEGIN MUTABLE GLOBAL STATE
pub static mut SURFACE: *const hw::Surface = ptr::null();
pub static mut ORIGINAL_ENGINE_FUNCS: MaybeUninit<EngineFuncs> = MaybeUninit::uninit();
pub static mut ORIGINAL_CLIENT_FUNCS: MaybeUninit<ClientFuncs> = MaybeUninit::uninit();
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
    _panel: panel::Hook,
}

impl Hook {
    fn new(modules: &Modules) -> Result<Hook, Error<'static>> {
        unsafe { SURFACE = modules.hw.create_interface::<hw::Surface>(hw::surface::INTERFACE)?; }
        info!("SURFACE = {:?}", unsafe { SURFACE });

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
    NumEntries = 131,
}

#[repr(C)]
#[derive(Clone)]
pub struct EngineFuncs {
    functions: [usize; EngineFuncsTable::NumEntries as usize]
}

impl EngineFuncs {
}

#[repr(usize)]
pub enum ClientFuncsTable {
    CreateMove = 15,
    NumEntries = 43,
}

#[repr(C)]
#[derive(Clone)]
pub struct ClientFuncs {
    functions: [usize; ClientFuncsTable::NumEntries as usize]
}

impl ClientFuncs {
    // void(*CL_CreateMove) (float frametime, struct usercmd_s *cmd, int active);
    pub fn create_move(&self, frame_time: f32, cmd: *mut UserCmd, active: i32) {
        type CreateMove = extern "C" fn(frame_time: f32, cmd: *mut UserCmd, active: i32);
        let address = self.functions[ClientFuncsTable::CreateMove as usize];
        let function: CreateMove = unsafe { mem::transmute(address) };
        function(frame_time, cmd, active);
    }

    pub fn hook(&mut self, function: ClientFuncsTable, hooked: usize) {
        self.functions[function as usize] = hooked;
    }
}

#[repr(C)]
pub struct UserCmd {
    lerp_msec: i16,
    msec: u8,
    view_angles: Vector,
    forward_move: f32,
    side_move: f32,
    up_move: f32,
    light_level: u8,
    buttons: u16,
    impulse: u8,
    weapon_select: u8,
    impact_index: i32,
    impact_position: Vector,
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

fn my_create_move(frame_time: f32, cmd: *mut UserCmd, active: i32) {
    unsafe {
        (*ORIGINAL_CLIENT_FUNCS.as_ptr()).create_move(frame_time, cmd, active)
    }

    info!("my_create_move()");
}

unsafe fn hook_client(client_funcs: *mut ClientFuncs) {
    (*client_funcs).hook(ClientFuncsTable::CreateMove, my_create_move as usize);
}

fn init_engine_and_client_funcs(hw: &Module) -> Result<(), Error<'static>> {
    let screen_fade = get_screen_fade_instruction(hw)?;

    unsafe {
        let engine_funcs: *const *const EngineFuncs = screen_fade.add(13).cast();
        let engine_funcs = engine_funcs.read_unaligned();
        memory::ptr_check(engine_funcs)?;
        info!("engine_funcs = {:?}", engine_funcs);
        ORIGINAL_ENGINE_FUNCS = MaybeUninit::new((*engine_funcs).clone());

        let client_funcs: *const *mut ClientFuncs = screen_fade.add(19).cast();
        let client_funcs = client_funcs.read_unaligned();
        memory::ptr_check(client_funcs)?;
        info!("client_funcs = {:?}", client_funcs);
        ORIGINAL_CLIENT_FUNCS = MaybeUninit::new((*client_funcs).clone());
        hook_client(client_funcs);
    }

    Ok(())
}

pub fn run() -> Result<(), Error<'static>> {
    let modules = Modules::new()?;
    let _hook = Hook::new(&modules)?;
    idle();
    Ok(())
}
