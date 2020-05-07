pub use surface::Surface;

pub mod surface {
    use std::mem;

    use winapi::ctypes::wchar_t;

    pub const INTERFACE: &str = "VGUI_Surface026";

    #[repr(usize)]
    pub enum Vtable {
        SetTextColor = 15,
        SetTextPos = 16,
        PrintText = 18,
        FlushText = 22,
        // NumEntries = 91,
    }

    #[repr(C)]
    pub struct Surface {
        vtable: *mut usize,
    }

    impl Surface {
        fn get_virtual_function_address(&self, function: Vtable) -> usize {
            unsafe { *self.vtable.add(function as usize) }
        }

        pub fn set_text_color(&self, r: i32, g: i32, b: i32, a: i32) {
            type SetTextColor = extern "fastcall" fn(this: &Surface, edx: usize, r: i32, g: i32, b: i32, a: i32);
            let set_text_color = self.get_virtual_function_address(Vtable::SetTextColor);
            let set_text_color: SetTextColor = unsafe { mem::transmute(set_text_color) };
            set_text_color(self, 0, r, g, b, a);
        }

        pub fn set_text_pos(&self, x: i32, y: i32) {
            type SetTextPos = extern "fastcall" fn(this: &Surface, edx: usize, x: i32, y: i32);
            let set_text_pos = self.get_virtual_function_address(Vtable::SetTextPos);
            let set_text_pos: SetTextPos = unsafe { mem::transmute(set_text_pos) };
            set_text_pos(self, 0, x, y);
        }

        pub fn print_text(&self, text: &[u16]) {
            type PrintText = extern "fastcall" fn(this: &Surface, edx: usize, text: *const wchar_t, len: i32);
            let print_text = self.get_virtual_function_address(Vtable::PrintText);
            let print_text: PrintText = unsafe { mem::transmute(print_text) };
            print_text(self, 0, text.as_ptr(), text.len() as i32);
        }

        pub fn flush_text(&self) {
            type FlushText = extern "fastcall" fn(this: &Surface, edx: usize);
            let flush_text = self.get_virtual_function_address(Vtable::FlushText);
            let flush_text: FlushText = unsafe { mem::transmute(flush_text) };
            flush_text(self, 0);
        }
    }
}