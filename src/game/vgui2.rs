pub use panel::Panel;

pub mod panel {
    use std::ffi::CStr;
    use std::mem;
    use std::os::raw::c_char;

    pub const INTERFACE: &str = "VGUI_Panel007";

    #[repr(usize)]
    pub enum Vtable {
        GetName = 36,
        PaintTraverse = 41,
        NumEntries = 60,
    }

    #[repr(C)]
    pub struct Panel {
        pub vtable: *mut [usize; Vtable::NumEntries as usize],
    }

    impl Panel {
        fn get_virtual_function_address(&self, function: Vtable) -> usize {
            let functions = unsafe { *self.vtable };
            functions[function as usize]
        }

        pub fn get_name<'p>(&self, panel: &'p Panel) -> Option<&'p CStr> {
            type GetName = extern "fastcall" fn(this: &Panel, edx: usize, panel: &Panel) -> *const c_char;
            let get_name = self.get_virtual_function_address(Vtable::GetName);
            let get_name: GetName = unsafe { mem::transmute(get_name) };
            
            let name = get_name(self, 0, panel);
            
            if name.is_null() {
                None
            } else {
                Some(unsafe { CStr::from_ptr(name) })
            }
        }
    }
}