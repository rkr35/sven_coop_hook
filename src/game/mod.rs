#![allow(clippy::type_complexity)]
#![allow(clippy::unseparated_literal_suffix)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod hw;
pub mod vgui2;

use std::ffi::CStr;
use std::iter;

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

impl user_msg_s {
    fn _iter(&self) -> impl Iterator<Item = &Self> {
        iter::successors(Some(self), |current| unsafe { current.next.as_ref() })
    }

    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.szName.as_ptr()) }
    }

    pub fn find<'n>(&mut self, name: &'n str) -> Option<*mut Self> {
        let mut messages = iter::successors(
            Some(self),
            |current| unsafe { current.next.as_mut() }
        );

        messages
            .find(|user_msg| user_msg.name().to_bytes() == name.as_bytes())
            .map(|user_msg| user_msg as *mut _)
    }
}