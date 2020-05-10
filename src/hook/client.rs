use crate::game::client::{ClientFuncs, ClientFuncsTable, UserCmd};
use crate::memory;

use log::info;

// BEGIN MUTABLE GLOBAL STATE
use crate::hook::ORIGINAL_CLIENT_FUNCS;
use crate::hook::PLAYER_MOVE;
// END MUTABLE GLOBAL STATE

pub struct Hook {
    client_funcs: *mut ClientFuncs,
}

impl Hook {
    pub fn new(client_funcs: *mut ClientFuncs) -> Self {
        unsafe {
            (*client_funcs).hook(ClientFuncsTable::CreateMove, my_create_move as usize);
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

fn bunny_hop(cmd: *mut UserCmd) {
    if memory::ptr_check(cmd).is_err() {
        return;
    }

    unsafe {
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
}

extern "C" fn my_create_move(frame_time: f32, cmd: *mut UserCmd, active: i32) {
    unsafe {
        ORIGINAL_CLIENT_FUNCS.as_ref().unwrap().create_move(frame_time, cmd, active)
    }

    bunny_hop(cmd);
}