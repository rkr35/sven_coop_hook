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
    pub buttons: u16,
    impulse: u8,
    weapon_select: u8,
    impact_index: i32,
    impact_position: Vector,
}

pub type Qboolean = i32;

#[repr(C)]
pub struct PlayerMove {
    player_index: i32,
    server: Qboolean,
    multiplayer: Qboolean,
    time: f32,
    frame_time: f32,
    forward: Vector,
    right: Vector,
    up: Vector,
    origin: Vector,
    angles: Vector,
    old_angles: Vector,
    velocity: Vector,
    move_dir: Vector,
    base_velocity: Vector,
    view_ofs: Vector,
    duck_time: f32,
    in_duck: Qboolean,
    time_step_sound: i32,
    step_left: i32,
    fall_velocity: f32,
    punch_angle: Vector,
    swim_time: f32,
    next_primary_attack: f32,
    effects: i32,
    pub flags: i32,
    use_hull: i32,
    gravity: f32,
    friction: f32,
    old_buttons: i32,
    water_jump_time: f32,
    dead: Qboolean,
    dead_flag: i32,
    spectator: i32,
    move_type: i32,
    on_ground: i32,
    pub water_level: i32,
}