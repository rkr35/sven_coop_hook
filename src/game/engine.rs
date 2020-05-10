use crate::game::client::Entity;

use std::mem;

#[repr(usize)]
pub enum EngineFuncsTable {
    GetLocalPlayer = 51,
    NumEntries = 131,
}

#[repr(C)]
#[derive(Clone)]
pub struct EngineFuncs {
    functions: [usize; EngineFuncsTable::NumEntries as usize]
}

impl EngineFuncs {
    // struct cl_entity_s			*( *GetLocalPlayer )		( void );
    pub fn get_local_player(&self) -> *const Entity {
        type GetLocalPlayer = extern "C" fn() -> *const Entity;
        let address = self.functions[EngineFuncsTable::GetLocalPlayer as usize];
        let function: GetLocalPlayer = unsafe { mem::transmute(address) };
        function()
    }
}
