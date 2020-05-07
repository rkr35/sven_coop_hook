pub use panel::Panel;

pub mod panel {
    pub const INTERFACE: &str = "VGUI_Panel007";

    #[repr(C)]
    pub struct Panel {
        pub vtable: *mut usize,
    }

    #[repr(usize)]
    pub enum Vtable {
        PaintTraverse = 41,
        NumEntries = 60,
    }
}