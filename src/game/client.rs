use std::mem;

use ultraviolet::Vec3 as Vector;

#[repr(usize)]
pub enum ClientFuncsTable {
    CreateMove = 14,
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

#[repr(C)]
pub struct PlayerMove {
    
}