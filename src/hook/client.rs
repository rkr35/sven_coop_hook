use crate::game::{cl_clientfuncs_s, cl_entity_s, entity_state_s, ref_params_s, usercmd_s};
use crate::single_thread_verifier;
use crate::yank::Yank;

use std::collections::HashMap;
use std::ffi::CStr;
use std::hash::BuildHasherDefault;
use std::os::raw::c_char;

use log::{info, warn};
use rustc_hash::FxHasher;

type FxBuilderHasher = BuildHasherDefault<FxHasher>;

// BEGIN MUTABLE GLOBAL STATE
use crate::hook::ORIGINAL_CLIENT_FUNCS;
use crate::hook::PLAYER_MOVE;
static mut ENTITIES: Option<HashMap<i32, *mut cl_entity_s, FxBuilderHasher>> = None;
// END MUTABLE GLOBAL STATE

pub struct Hook {
    client_funcs: *mut cl_clientfuncs_s,
}

impl Hook {
    pub fn new(client_funcs: *mut cl_clientfuncs_s) -> Self {
        unsafe {
            // Used by hooks. Must initialize before hooking.
            ENTITIES = Some(HashMap::default());

            (*client_funcs).CL_CreateMove = Some(my_create_move);
            (*client_funcs).V_CalcRefdef = Some(my_calc_ref_def);
            (*client_funcs).HUD_AddEntity = Some(my_hud_add_entity);
            (*client_funcs).HUD_ProcessPlayerState = Some(my_hud_process_player_state);
        }

        Self {
            client_funcs
        }
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

// Idea.
// We don't have to worry about thread-synchronization problems.
// These 3 client functions run on the same thread.
// There is no context-switching among these functions (and likely all the client functions).
// Use a HashSet where the elements are keyed by their *mut cl_entity_s.
// Add entities if you haven't already (HashSet takes care of existing entry requirement).
// Remove entities when you detect they are no longer valid.
// TODO: How do you check if an entity is valid if you only have a dangling pointer to that entity?
// Replaced with HashMap to key by index (not using array; sparse elements).
// Probably true: ENTITIES[i] is not a dangling pointer if initialized at least once within the
// map. 
// Max entities: 8192 ?
// 
unsafe extern "C" fn my_hud_add_entity(typ: i32, ent: *mut cl_entity_s, modelname: *const c_char) -> i32 {
    single_thread_verifier::assert();

    let original = ORIGINAL_CLIENT_FUNCS.yank_ref().HUD_AddEntity.yank();
    let ret = original(typ, ent, modelname);

    let index = (*ent).index;

    if index == 0 {
        return ret;
    }

    if let Some(prev_ent) = ENTITIES.yank_mut().insert(index, ent) {
        if prev_ent != ent {
            warn!(
                "Replaced entity with index={} and modelname={:?} with entity that has modelname={:?}",
                index,
                CStr::from_ptr({
                    let model = (*prev_ent).model;
                    (*model).name.as_ptr()
                }),
                CStr::from_ptr(modelname)
            );
        }
    } else {
        info!("Added entity with index={} and modelname={:?}", (*ent).index, CStr::from_ptr(modelname));
    }

    ret

}

unsafe extern "C" fn my_hud_process_player_state(dst: *mut entity_state_s, src: *const entity_state_s) {
    single_thread_verifier::assert();

    let original = ORIGINAL_CLIENT_FUNCS.yank_ref().HUD_ProcessPlayerState.yank();
    original(dst, src)
}