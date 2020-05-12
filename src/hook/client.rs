use crate::game::{cl_clientfuncs_s, ref_params_s, usercmd_s};
use crate::memory;
use crate::yank::Yank;

use log::info;

// BEGIN MUTABLE GLOBAL STATE
use crate::hook::ORIGINAL_CLIENT_FUNCS;
use crate::hook::PLAYER_MOVE;
// END MUTABLE GLOBAL STATE

pub struct Hook {
    client_funcs: *mut cl_clientfuncs_s,
}

impl Hook {
    pub fn new(client_funcs: *mut cl_clientfuncs_s) -> Self {
        unsafe {
            (*client_funcs).CL_CreateMove = Some(my_create_move);
            (*client_funcs).V_CalcRefdef = Some(my_calc_ref_def);
        }

        Self {
            client_funcs
        }
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        unsafe {
            // TODO:
            // Investigate race condition w/ client hooks accessing ORIGINAL_CLIENT_FUNCS?
            // Shouldn't matter if the generated assembly always resolves the absolute address of
            // original client functions, i.e., doesn't use the ORIGINAL_CLIENT_FUNCS to calculate
            // the address.
            *self.client_funcs = ORIGINAL_CLIENT_FUNCS.take().unwrap();
        }

        info!("Client hook dropped.");
    }
}

unsafe fn bunny_hop(cmd: *mut usercmd_s) {
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
    let original = ORIGINAL_CLIENT_FUNCS.as_ref().yank().CL_CreateMove.yank();
    original(frame_time, cmd, active);

    if memory::ptr_check(cmd).is_err() {
        return;
    }

    bunny_hop(cmd);
}

// void(*V_CalcRefdef) (struct ref_params_s *pparams);
unsafe extern "C" fn my_calc_ref_def(params: *mut ref_params_s) {
    let original = ORIGINAL_CLIENT_FUNCS.as_ref().yank().V_CalcRefdef.yank();
    original(params);
    
    if memory::ptr_check(params).is_err() {
        return;
    }
}