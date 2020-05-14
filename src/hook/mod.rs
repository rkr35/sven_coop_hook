use crate::game::{cl_clientfuncs_s, cl_enginefuncs_s, playermove_s, user_msg_s};
use crate::game::hw;
use crate::idle;
use crate::memory;
use crate::module::{self, Module};

use std::mem;
use std::ptr;

use log::{error, info};
use thiserror::Error;

mod client;
mod panel;
mod user_msg;

// BEGIN MUTABLE GLOBAL STATE
pub static mut SURFACE: *const hw::Surface = ptr::null();
pub static mut ENGINE_FUNCS: *const cl_enginefuncs_s = ptr::null();
pub static mut ORIGINAL_CLIENT_FUNCS: Option<cl_clientfuncs_s> = None;
pub static mut PLAYER_MOVE: *const playermove_s = ptr::null();
pub static mut USER_MSG: *const user_msg_s = ptr::null(); 
// END MUTABLE GLOBAL STATE

type Result<T> = std::result::Result<T, Error<'static>>;

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

    #[error("memory error: {0}")]
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
    _user_msg: user_msg::Hook,
}

impl Hook {
    fn new(modules: &Modules) -> Result<Hook> {
        let screen_fade = get_screen_fade_instruction(&modules.hw)?;

        unsafe {
            init_surface(&modules.hw)?;
            init_engine_funcs(screen_fade)?;
            init_player_move(screen_fade)?;
            init_user_msg()?;
        };

        Ok(Hook {
            _client: unsafe { hook_client_funcs(screen_fade)? },
            _panel: panel::Hook::new(&modules.vgui2)?,
            _user_msg: user_msg::Hook::new(),
        })
    }
}

struct Modules {
    hw: Module,
    vgui2: Module,
}

impl Modules {
    fn new() -> Result<Modules> {
        Ok(Modules {
            hw: Module::from("hw.dll")?,
            vgui2: Module::from("vgui2.dll")?,
        })
    }
}

fn get_screen_fade_instruction(hw: &Module) -> Result<*const u8> {
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

unsafe fn init_surface(hw: &Module) -> Result<()> {
    SURFACE = hw.create_interface::<hw::Surface>(hw::surface::INTERFACE)?;
    info!("SURFACE = {:?}", SURFACE);
    Ok(())
}

unsafe fn init_engine_funcs(screen_fade: *const u8) -> Result<()> {
    let engine_funcs: *const *const cl_enginefuncs_s = screen_fade.add(13).cast();
    ENGINE_FUNCS = engine_funcs.read_unaligned();
    memory::ptr_check(ENGINE_FUNCS)?;
    info!("ENGINE_FUNCS = {:?}", ENGINE_FUNCS);
    Ok(())
}

unsafe fn init_player_move(screen_fade: *const u8) -> Result<()> {
    let player_move: *const *const playermove_s = screen_fade.add(36).cast();
    PLAYER_MOVE = player_move.read_unaligned();
    memory::ptr_check(PLAYER_MOVE)?;
    info!("PLAYER_MOVE = {:?}", PLAYER_MOVE);
    Ok(())
}

unsafe fn init_user_msg() -> Result<()> {
    /*
        FF 74 24 08 FF 74 24 08
        hook_user_msg:  push dword ptr ss:[esp+8]
                        push dword ptr ss:[esp+8]
    */
    let hook_user_msg: *const u8 = mem::transmute((*ENGINE_FUNCS).pfnHookUserMsg.unwrap());
    
    /*
        E8 83 25 01 00
        call_inner: call hw.421C800
    */
    let call_inner_operand: *const *const u8 = hook_user_msg.add(9).cast();

    /*
        next_instruction:   we only care about the absolute address to resolve the jump of previous call.
    */
    let next_instruction = call_inner_operand.add(1) as usize;

    /*
        53 8B 5C 24 08 55 8B 6C 24 10 56
        inner_function: push ebx
                        mov ebx,dword ptr ss:[esp+8]
                        push ebp
                        mov ebp,dword ptr ss:[esp+10]
                        push esi
    */    
    let inner_function = call_inner_operand.read_unaligned().add(next_instruction);

    /*
        8B 35 50 8D B5 04
        load_head:  mov esi,dword ptr ds:[4B58D50]
    */
    let head_of_user_msg_linked_list: *const *const user_msg_s = inner_function.add(13).cast();
    USER_MSG = head_of_user_msg_linked_list.read_unaligned();
    memory::ptr_check(USER_MSG)?;
    info!("USER_MSG = {:?}", USER_MSG);
    Ok(())
}

unsafe fn hook_client_funcs(screen_fade: *const u8) -> Result<client::Hook> {
    let client_funcs: *const *mut cl_clientfuncs_s = screen_fade.add(19).cast();
    let client_funcs = client_funcs.read_unaligned();
    memory::ptr_check(client_funcs)?;
    info!("client_funcs = {:?}", client_funcs);
    ORIGINAL_CLIENT_FUNCS = (*client_funcs).clone().into();
    Ok(client::Hook::new(client_funcs))
}

pub fn run() -> Result<()> {
    let modules = Modules::new()?;
    let _hook = Hook::new(&modules)?;
    idle();
    Ok(())
}
