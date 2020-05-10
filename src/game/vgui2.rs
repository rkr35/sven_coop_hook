pub use panel::Panel;

pub mod panel {
    use sven_coop_hook_macros::vtable;

    use std::ffi::CStr;
    use std::os::raw::c_char;

    pub const INTERFACE: &str = "VGUI_Panel007";
    pub const NUM_VTABLE_ENTRIES: usize = 60;

    #[repr(C)]
    pub struct Panel {
        pub vtable: *mut [usize; NUM_VTABLE_ENTRIES],
    }

    impl Panel {
        vtable! {
            36: get_name_impl(panel: &Panel) -> *const c_char,
        }

        pub fn get_name<'p>(&self, panel: &'p Panel) -> Option<&'p CStr> {
            let name = self.get_name_impl(panel);
            
            if name.is_null() {
                None
            } else {
                Some(unsafe { CStr::from_ptr(name) })
            }
        }
    }
}