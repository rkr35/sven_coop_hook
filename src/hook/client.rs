use crate::client::{ClientFuncs, ClientFuncsTable, UserCmd};

use log::info;

// BEGIN MUTABLE GLOBAL STATE
use crate::hook::ORIGINAL_CLIENT_FUNCS;
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
            *(self.client_funcs) = (*ORIGINAL_CLIENT_FUNCS.as_ptr()).clone()
        }

        info!("Client hook dropped.");
    }
}

extern "C" fn my_create_move(frame_time: f32, cmd: *mut UserCmd, active: i32) {
    unsafe {
        (*ORIGINAL_CLIENT_FUNCS.as_ptr()).create_move(frame_time, cmd, active)
    }

    info!("fps = {}", 1.0 / frame_time);
}