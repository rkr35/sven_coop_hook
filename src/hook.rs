use crate::idle;
use crate::module::{Error as ModuleError, Module};
use crate::vgui::Panel;
use std::mem;

use log::{error, info};
use winapi::um::{
    memoryapi::VirtualProtect,
    winnt::PAGE_EXECUTE_READWRITE,
};

struct Hook<'a> {
    panel: &'a mut Panel,
}

impl<'a> Drop for Hook<'a> {
    fn drop(&mut self) {
        info!("Hook dropped.");
    }
}

struct Modules {
    hw: Module<'static>,
    vgui2: Module<'static>,
}

impl Modules {
    fn new() -> Result<Self, ModuleError<'static>> {
        Ok(Self {
            hw: Module::from("hw.dll")?,
            vgui2: Module::from("vgui2.dll")?,
        })
    }
}

static mut OLD_PAINT_TRAVERSE: usize = 0;

// 0x1ef68980 {vtable=vgui2.dll!0x52f9c148 {1391708272} }
extern "fastcall" fn my_paint_traverse(this: usize, edx: usize, panel: usize, force_repaint: bool, allow_force: bool) {
    type PaintTraverseFn = extern "fastcall" fn(usize, usize, usize, bool, bool);
    let original = unsafe { mem::transmute::<usize, PaintTraverseFn>(OLD_PAINT_TRAVERSE) };
    original(this, edx, panel, force_repaint, allow_force);
    info!("Called original.");
}

fn hook_and_idle(modules: &Modules) -> Result<(), ModuleError> {
    let panel = modules.vgui2.create_interface::<Panel>("VGUI_Panel007")?;
    info!("panel = {:#x?}", panel as *const _);
    unsafe {
        let ptr = panel.vtable.add(41);
        OLD_PAINT_TRAVERSE = *ptr;
        info!("OLD_PAINT_TRAVERSE = {:#x?}", OLD_PAINT_TRAVERSE);
        let mut old_protect = 0;
        VirtualProtect(ptr.cast(), 4, PAGE_EXECUTE_READWRITE, &mut old_protect);
        *ptr = my_paint_traverse as usize;
        VirtualProtect(ptr.cast(), 4, old_protect, &mut old_protect);
    }
    let _hook = Hook { panel };
    idle();
    Ok(())
}

pub fn run() {
    match Modules::new() {
        Ok(modules) => if let Err(e) = hook_and_idle(&modules) {
            error!("Hook error: {:?}", e);
            idle();
        },

        Err(e) => {
            error!("Modules error: {:?}", e);
            idle();
        }
    }
}