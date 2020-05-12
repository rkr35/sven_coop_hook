#![allow(clippy::type_complexity)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod hw;
pub mod vgui2;

use ultraviolet::Vec3 as vec3_t;
include!(concat!(env!("OUT_DIR"), "/sdk.rs"));

impl Clone for cl_clientfuncs_s {
    fn clone(&self) -> Self {
        unsafe { std::ptr::read(self) }
    }
}