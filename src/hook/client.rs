use crate::game::client::{ClientFuncs, ClientFuncsTable, RefParams, UserCmd};
use crate::memory;

use log::info;
use ultraviolet::Vec3 as Vector;

// BEGIN MUTABLE GLOBAL STATE
use crate::hook::ORIGINAL_CLIENT_FUNCS;
use crate::hook::PLAYER_MOVE;
static mut RECOIL_ANGLE: Vector = Vector { x: 0.0, y: 0.0, z: 0.0 };
// END MUTABLE GLOBAL STATE

pub struct Hook {
    client_funcs: *mut ClientFuncs,
}

impl Hook {
    pub fn new(client_funcs: *mut ClientFuncs) -> Self {
        unsafe {
            (*client_funcs).hook(ClientFuncsTable::CreateMove, my_create_move as usize);
            (*client_funcs).hook(ClientFuncsTable::CalcRefDef, my_calc_ref_def as usize);
        }

        Self {
            client_funcs
        }
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        unsafe {
            *self.client_funcs = ORIGINAL_CLIENT_FUNCS.as_ref().unwrap().clone();
        }

        info!("Client hook dropped.");
    }
}

unsafe fn bunny_hop(cmd: *mut UserCmd) {
    const IN_JUMP: u16 = 1 << 1;
    const FL_ONGROUND: i32 = 1 << 9;

    if (*cmd).buttons & IN_JUMP != IN_JUMP {
        return;
    }

    (*cmd).buttons &= !IN_JUMP;

    let on_ground = (*PLAYER_MOVE).flags & FL_ONGROUND == FL_ONGROUND;
    
    if on_ground || (*PLAYER_MOVE).water_level >= 2 {
        (*cmd).buttons |= IN_JUMP;
    }
}

unsafe extern "C" fn my_create_move(frame_time: f32, cmd: *mut UserCmd, active: i32) {
    ORIGINAL_CLIENT_FUNCS.as_ref().unwrap().create_move(frame_time, cmd, active);

    if memory::ptr_check(cmd).is_err() {
        return;
    }

    bunny_hop(cmd);
    (*cmd).view_angles = Vector::zero();
}

// void(*V_CalcRefdef) (struct ref_params_s *pparams);
unsafe extern "C" fn my_calc_ref_def(params: *mut RefParams) {
    ORIGINAL_CLIENT_FUNCS.as_ref().unwrap().calc_ref_def(params);
    
    if memory::ptr_check(params).is_err() {
        return;
    }
}