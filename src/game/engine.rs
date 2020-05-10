#[repr(usize)]
pub enum EngineFuncsTable {
    NumEntries = 131,
}

#[repr(C)]
#[derive(Clone)]
pub struct EngineFuncs {
    functions: [usize; EngineFuncsTable::NumEntries as usize]
}

impl EngineFuncs {
}
