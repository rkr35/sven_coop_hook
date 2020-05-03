#[repr(C)]
pub struct Panel {
    pub vtable: *mut usize, // 0x683ac2e4
}

// vtable entry address: panel.vtable + 0x2c
// function pointer = (*vtable entry address) cast

// vtable entry address: vtable.add(11) = 0x683AC310
// *vtable.add(11) = *0x683AC310 = 0x6834D620 