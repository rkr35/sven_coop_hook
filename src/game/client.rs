#![allow(dead_code)]
use sven_coop_hook_macros::functions;

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
    functions! {
        ClientFuncsTable::CreateMove pub create_move(frame_time: f32, cmd: *mut UserCmd, active: i32),
        ClientFuncsTable::CalcRefDef pub calc_ref_def(params: *mut RefParams),
    }

    pub fn hook(&mut self, function: ClientFuncsTable, hooked: usize) {
        self.functions[function as usize] = hooked;
    }
}

#[repr(C)]
pub struct UserCmd {
    lerp_msec: i16,
    msec: u8,
    pub view_angles: Vector,
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
    pub health: i32,
    crosshair_angle: Vector,
    view_size: f32,
    pub punch_angle: Vector,
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


#[repr(C)]
pub struct EntityState {
    entity_type: i32,
    number: i32,
    msg_time: f32,
    message_num: i32,
    origin: Vector,
    angles: Vector,
    model_index: i32,
    sequence: i32,
    frame: f32,
    color_map: i32,
    skin: u16,
    solid: u16,
    effects: i32,
    scale: f32,
    eflags: u8,
    render_mode: i32,
    render_amt: i32,
    render_color: [u8; 3],
    render_fx: i32,
    move_type: i32,
    anim_time: f32,
    frame_rate: f32,
    body: i32,
    controller: [u8; 4],
    blending: [u8; 4],
    velocity: Vector,
    mins: Vector,
    maxs: Vector,
    aim_ent: i32,
    owner: i32,
    friction: f32,
    gravity: f32,
    team: i32,
    player_class: i32,
    health: i32,
    spectator: Qboolean,
    weapon_model: i32,
    gait_sequence: i32,
    base_velocity: Vector,
    use_hull: i32,
    old_buttons: i32,
    on_ground: i32,
    step_left: i32,
    fall_velocity: f32,
    fov: f32,
    weapon_anim: i32,
    start_pos: Vector,
    end_pos: Vector,
    impact_time: f32,
    start_time: f32,
    iuser1: i32,
    iuser2: i32,
    iuser3: i32,
    iuser4: i32,
    fuser1: f32,
    fuser2: f32,
    fuser3: f32,
    fuser4: f32,
    vuser1: Vector,
    vuser2: Vector,
    vuser3: Vector,
    vuser4: Vector,
}

#[repr(C)]
pub struct PositionHistory {
    anim_time: f32,
    origin: Vector,
    angles: Vector,
}

#[repr(C)]
pub struct Mouth {
    mouth_open: u8,
    snd_count: u8,
    snd_avg: i32,
}

#[repr(C)]
pub struct LatchedVars {
    prev_anim_time: f32,
    sequence_time: f32,
    prev_seq_blending: [u8; 2],
    prev_origin: Vector,
    prev_angles: Vector,
    prev_sequence: i32,
    prev_frame: f32,
    prev_controller: [u8; 4],
    prev_blending: [u8; 2],
}

// https://doc.rust-lang.org/reference/type-layout.html#the-c-representation
// "Note: The enum representation in C is implementation defined, so this is really a 'best guess'.
//      In particular, this may be incorrect when the C code of interest is compiled with certain flags."
#[repr(C)]
pub enum ModType {
    Brush,
    Sprite,
    Alias,
    Studio,
}

#[repr(C)]
pub enum SyncType {
    Sync,
    Rand,
}

pub const MAX_MODEL_NAME: usize = 64;

#[repr(C)]
pub struct Model {
    name: [c_char; MAX_MODEL_NAME],
    needload: Qboolean,
    mod_type: ModType,
    numframes: i32,
    synctype: SyncType,
    flags: i32,
}

pub const NUM_AMBIENTS: usize = 4;

#[repr(C)]
pub struct Mleaf {
    contents: i32,
    visframe: i32,
    minmaxs: [u16; 6],
    parent: usize,//*const Mnode,
    compressed_vis: *const u8,
    efrags: *const Efrag,
    firstmarksurface: usize,//*const *const Msurface,
    nummarksurfaces: i32,
    key: i32,
    ambient_sound_level: [u8; NUM_AMBIENTS],
}

#[repr(C)]
pub struct Efrag {
    leaf: *const Mleaf,
    leaf_next: *const Efrag,
    entity: *const Entity,
    ent_next: *const Efrag,
}

pub const HISTORY_MAX: usize = 64;

#[repr(C)]
pub struct Entity {
    index: i32,
    pub player: Qboolean,
    base_line: EntityState,
    prev_state: EntityState,
    cur_state: EntityState,
    current_position: i32,
    position_history: [PositionHistory; HISTORY_MAX],
    mouth: Mouth,
    latched: LatchedVars,
    last_move: f32,
    origin: Vector,
    angles: Vector,
    attachment: [Vector; 4],
    trivial_accept: i32,
    model: *const Model,
    efrag: *const Efrag,
    top_node: usize,//*const Mnode,
    sync_base: f32,
    vis_frame: i32,
    floor_color: [u32; 4],
}