use std::mem;

use thiserror::Error;
use winapi::um::{memoryapi::VirtualProtect, winnt::PAGE_EXECUTE_READWRITE};

#[derive(Error, Debug)]
pub enum Error {
    #[error("null pointer")]
    NullPointer,

    #[error("the pointer is unaligned; \
             the pointer needs to be offset by {num_bytes_needed_to_align_ptr} bytes \
             to align {address:#x} to the required alignment of {required_alignment} bytes")]
    UnalignedPointer {
        address: usize,
        required_alignment: usize,
        num_bytes_needed_to_align_ptr: usize,
    },
}

pub struct Patch<T> where T: Copy + Clone {
    address: *mut T,
    pub old_value: T,
}

impl<T> Patch<T> where T: Copy + Clone {
    pub unsafe fn new(address: *mut T, new_value: T) -> Result<Self, Error> {
        Ok(Self {
            address,
            old_value: Self::patch(address, new_value)?,
        })
    }

    unsafe fn patch(pointer: *mut T, new_value: T) -> Result<T, Error> {
        ptr_check(pointer)?;

        let old_value;
        let mut old_protection = 0;

        // Save the previous value at this address.
        old_value = *pointer;

        // Change page protection and write new value.
        VirtualProtect(pointer.cast(), mem::size_of::<T>(), PAGE_EXECUTE_READWRITE, &mut old_protection);
        *pointer = new_value;
        VirtualProtect(pointer.cast(), mem::size_of::<T>(), old_protection, &mut old_protection);

        Ok(old_value)
    }

    pub unsafe fn restore(&self) {
        let _ = Self::patch(self.address, self.old_value);
    }
}

pub fn ptr_check<T>(p: *const T) -> Result<(), Error> {
    if p.is_null() {
        return Err(Error::NullPointer);
    }

    let required_alignment = mem::align_of::<T>();
    let num_bytes_needed_to_align_ptr = p.align_offset(required_alignment);

    if num_bytes_needed_to_align_ptr > 0 {
        return Err(Error::UnalignedPointer {
            address: p as usize,
            required_alignment,
            num_bytes_needed_to_align_ptr,
        });
    }
    
    Ok(())
}