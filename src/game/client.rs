use std::mem;
use std::os::raw::c_char;

use ultraviolet::Vec3 as Vector;

#[repr(usize)]
pub enum ClientFuncsTable {
    CreateMove = 14,
    CalcRefDef = 19,
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

    // void(*V_CalcRefdef) (struct ref_params_s *pparams);
    pub fn calc_ref_def(&self, params: *mut RefParams) {
        type CalcRefDef = extern "C" fn(params: *mut RefParams);
        let address = self.functions[ClientFuncsTable::CalcRefDef as usize];
        let function: CalcRefDef = unsafe { mem::transmute(address) };
        function(params);
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

#[repr(C)]
pub struct MoveVars {
    gravity: f32,           // Gravity for map
    stop_speed: f32,         // Deceleration when not moving
    max_speed: f32,          // Max allowed speed
    spectator_maxspeed: f32,
    accelerate: f32,        // Acceleration factor
    air_accelerate: f32,     // Same for when in open air
    water_accelerate: f32,   // Same for when in water
    friction: f32,          
    edge_friction: f32,	   // Extra friction near dropofs 
    water_friction: f32,     // Less in water
    ent_gravity: f32,        // 1.0
    bounce: f32,            // Wall bounce value. 1.0
    step_size: f32,          // sv_stepsize;
    max_velocity: f32,       // maximum server velocity.
    z_max: f32,			   // Max z-buffer range (for GL)
    wave_height: f32,		   // Water wave height (for GL)
    footsteps: Qboolean,        // Play footstep sounds
    sky_name: [c_char; 32],	   // Name of the sky map
    roll_angle: f32,
    roll_speed: f32,
    sky_color_r: f32,			// Sky color
    sky_color_g: f32,			// 
    sky_color_b: f32,			//
    sky_vec_x: f32,			// Sky vector
    sky_vec_y: f32,			// 
    sky_vec_z: f32,			// 
}

#[repr(C)]
pub struct RefParams {
    view_org: Vector,
    pub view_angles: Vector,
    forward: Vector,
    right: Vector,
    up: Vector,
    frame_time: f32,
    time: f32,
    intermission: i32,
    paused: i32,
    spectator: i32,
    on_ground: i32,
    water_level: i32,
    sim_vel: Vector,
    sim_org: Vector,
    view_height: Vector,
    ideal_pitch: f32,
    cl_view_angles: Vector,
    health: i32,
    crosshair_angle: Vector,
    view_size: f32,
    punch_angle: Vector,
    max_clients: i32,
    view_entity: i32,
    player_num: i32,
    max_entities: i32,
    demo_playback: i32,
    hardware: i32,
    smoothing: i32,
    cmd: *const UserCmd,
    move_vars: *const MoveVars,
    viewport: [i32; 4],
    next_view: i32,
    only_client_draw: i32,
}