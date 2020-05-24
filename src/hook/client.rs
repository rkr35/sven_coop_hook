use crate::game::{cl_clientfuncs_s, cl_entity_s, entity_state_s, ref_params_s, usercmd_s};
use crate::single_thread_verifier;
use crate::yank::Yank;

use std::collections::HashSet;
use std::ffi::CStr;
use std::hash::BuildHasherDefault;
use std::os::raw::c_char;
use std::ptr;

use bstr::BStr;
use log::info;
use static_assertions as sa;

const MAX_ENTITIES: i32 = 8192;
sa::const_assert!(MAX_ENTITIES > 0);

// BEGIN MUTABLE GLOBAL STATE
use crate::hook::ORIGINAL_CLIENT_FUNCS;
use crate::hook::PLAYER_MOVE;
static mut ENTITIES: [*mut cl_entity_s; MAX_ENTITIES as usize]  = [ptr::null_mut(); MAX_ENTITIES as usize];
// END MUTABLE GLOBAL STATE

pub struct Hook {
    client_funcs: *mut cl_clientfuncs_s,
}

impl Hook {
    pub fn new(client_funcs: *mut cl_clientfuncs_s) -> Self {
        unsafe {
            (*client_funcs).CL_CreateMove = Some(my_create_move);
            (*client_funcs).V_CalcRefdef = Some(my_calc_ref_def);
            (*client_funcs).HUD_AddEntity = Some(my_hud_add_entity);
            (*client_funcs).HUD_ProcessPlayerState = Some(my_hud_process_player_state);
            (*client_funcs).HUD_Frame = Some(my_hud_frame);
        }

        Self { client_funcs }
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        unsafe {
            *self.client_funcs = ORIGINAL_CLIENT_FUNCS.as_ref().cloned().unwrap();
        }

        info!("Client hook dropped.");
    }
}

unsafe fn _bunny_hop(cmd: *mut usercmd_s) {
    const IN_JUMP: u16 = 1 << 1;
    const FL_ONGROUND: i32 = 1 << 9;

    if (*cmd).buttons & IN_JUMP != IN_JUMP {
        return;
    }

    (*cmd).buttons &= !IN_JUMP;

    let on_ground = (*PLAYER_MOVE).flags & FL_ONGROUND == FL_ONGROUND;

    if on_ground || (*PLAYER_MOVE).waterlevel >= 2 {
        (*cmd).buttons |= IN_JUMP;
    }
}

unsafe extern "C" fn my_create_move(frame_time: f32, cmd: *mut usercmd_s, active: i32) {
    single_thread_verifier::assert();

    let original = ORIGINAL_CLIENT_FUNCS.yank_ref().CL_CreateMove.yank();
    original(frame_time, cmd, active);

    if cmd.is_null() {
        return;
    }
}

// void(*V_CalcRefdef) (struct ref_params_s *pparams);
unsafe extern "C" fn my_calc_ref_def(params: *mut ref_params_s) {
    single_thread_verifier::assert();

    let original = ORIGINAL_CLIENT_FUNCS.yank_ref().V_CalcRefdef.yank();
    original(params);

    if params.is_null() {
        return;
    }
}

unsafe fn manage_entity(ent: *mut cl_entity_s, modelname: *const c_char) {
    if ent.is_null() || modelname.is_null() {
        return;
    }

    let index = (*ent).index;

    if index <= 0 || index >= MAX_ENTITIES {
        return;
    }

    // We already check for non-positive indices above.
    // We are casting from a smaller positive domain to a larger positive domain,
    // so the cast here from i32 -> usize is lossless.
    #[allow(clippy::cast_sign_loss)]
    let index = index as usize;

    let name = CStr::from_ptr(modelname).to_bytes();

    if !is_model(name) {
        return;
    }

    let name: &BStr = name
        .rsplit(|&byte| byte == b'/')
        .next()
        .yank()
        .into();

    let slot = ENTITIES.get_mut(index).yank();

    if (*ent).is_alive() {
        if slot.is_null() {
            *slot = ent;
            info!("Added {:?} ({:?}).", ent, name);
        }
    } else if !slot.is_null() {
        *slot = ptr::null_mut();
        info!("Removed {:?} ({:?}).", ent, name);
    }
}

unsafe extern "C" fn my_hud_add_entity(
    typ: i32,
    ent: *mut cl_entity_s,
    modelname: *const c_char,
) -> i32 {
    single_thread_verifier::assert();

    manage_entity(ent, modelname);

    let original = ORIGINAL_CLIENT_FUNCS.yank_ref().HUD_AddEntity.yank();
    original(typ, ent, modelname)
}

unsafe extern "C" fn my_hud_process_player_state(
    dst: *mut entity_state_s,
    src: *const entity_state_s,
) {
    single_thread_verifier::assert();

    let original = ORIGINAL_CLIENT_FUNCS.yank_ref().HUD_ProcessPlayerState.yank();
    original(dst, src)
}

unsafe extern "C" fn my_hud_frame(time: f64) {
    single_thread_verifier::assert();

    let original = ORIGINAL_CLIENT_FUNCS.yank_ref().HUD_Frame.yank();
    original(time);
}

fn is_model(name: &[u8]) -> bool {
    const MODELS_SUFFIX: [u8; 4] = *b".mdl";
    name.ends_with(&MODELS_SUFFIX)
}