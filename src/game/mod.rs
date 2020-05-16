#![allow(clippy::type_complexity)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod hw;
pub mod vgui2;

use std::ffi::CStr;

use ultraviolet::Vec3 as vec3_t;
include!(concat!(env!("OUT_DIR"), "/sdk.rs"));

impl Clone for cl_clientfuncs_s {
    fn clone(&self) -> Self {
        unsafe { std::ptr::read(self) }
    }
}

impl cl_entity_s {
    pub fn name(&self) -> Option<&CStr> {
        let model = self.model;
        
        if model.is_null() {
            return None;
        }
        
        Some(unsafe {
            let name = (*model).name.as_ptr();
            CStr::from_ptr(name)
        })
    }
}