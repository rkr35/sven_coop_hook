use std::mem;

use winapi::um::{
    memoryapi::VirtualProtect,
    winnt::PAGE_EXECUTE_READWRITE,
};

pub struct Patch<T> where T: Copy + Clone {
    address: *mut T,
    pub old_value: T,
}

impl<T> Patch<T> where T: Copy + Clone {
    pub fn new(address: *mut T, new_value: T) -> Option<Self> {
        Some(Self {
            address,
            old_value: Self::patch(address, new_value)?
        })
    }

    fn patch(pointer: *mut T, new_value: T) -> Option<T> {
        if pointer.is_null() {
            return None;
        }

        let old_value;
        let mut old_protection = 0;

        unsafe {
            // Save the previous value at this address.
            old_value = *pointer;

            // Change page protection and write new value.
            VirtualProtect(pointer.cast(), mem::size_of::<T>(), PAGE_EXECUTE_READWRITE, &mut old_protection);
            *pointer = new_value;
            VirtualProtect(pointer.cast(), mem::size_of::<T>(), old_protection, &mut old_protection);
        }

        Some(old_value)
    }
}

impl<T> Drop for Patch<T> where T: Copy + Clone {
    fn drop(&mut self) {
        Self::patch(self.address, self.old_value);
    }
}