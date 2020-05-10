use crate::game::client::Entity;
use sven_coop_hook_macros::functions;

const NUM_ENGINE_FUNCS: usize = 131;

#[repr(C)]
#[derive(Clone)]
pub struct EngineFuncs {
    functions: [usize; NUM_ENGINE_FUNCS]
}

impl EngineFuncs {
    functions! {
        36: pub get_max_clients() -> i32,
        51: get_local_player() -> *const Entity,
    }
}
