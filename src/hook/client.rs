use crate::client::{ClientFuncs, ClientFuncsTable, UserCmd};

use log::info;

// BEGIN MUTABLE GLOBAL STATE
use crate::hook::ORIGINAL_CLIENT_FUNCS;
// END MUTABLE GLOBAL STATE

pub struct Hook {

}

impl Hook {
    pub unsafe fn new(client_funcs: *mut ClientFuncs) -> Self {
        Self {

        }
    }
}

impl Drop for Hook {
    fn drop(&mut self) {

    }
}

pub unsafe fn hook(client_funcs: *mut ClientFuncs) {
    (*client_funcs).hook(ClientFuncsTable::CreateMove, my_create_move as usize);
}

extern "C" fn my_create_move(frame_time: f32, cmd: *mut UserCmd, active: i32) {
    unsafe {
        (*ORIGINAL_CLIENT_FUNCS.as_ptr()).create_move(frame_time, cmd, active)
    }

    info!("my_create_move()");
}