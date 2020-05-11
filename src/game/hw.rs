pub use surface::Surface;

pub mod surface {
    use sven_coop_hook_macros::vtable;
    
    use winapi::ctypes::wchar_t;

    pub const INTERFACE: &str = "VGUI_Surface026";
    pub const NUM_VTABLE_ENTRIES: usize = 91;

    #[repr(C)]
    pub struct Surface {
        vtable: *mut [usize; NUM_VTABLE_ENTRIES],
    }

    impl Surface {
        vtable! {
            15 pub set_text_color(r: i32, g: i32, b: i32, a: i32),
            16 pub set_text_pos(x: i32, y: i32),
            18 print_text_impl(text: *const wchar_t, len: i32),
        }

        pub fn print_text(&self, text: &[u16]) {
            self.print_text_impl(text.as_ptr(), text.len() as i32);
        }
    }
}