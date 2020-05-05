use crate::idle;
use crate::memory::Patch;
use crate::module::{Error as ModuleError, Module};
use crate::vgui::Panel;
use std::mem;

use log::{error, info};

struct Hook<'a> {
    _panel: &'a mut Panel,
    _paint_traverse_hook: Patch<usize>,
}

impl<'a> Drop for Hook<'a> {
    fn drop(&mut self) {
        info!("Hook dropped.");
    }
}

struct Modules {
    _hw: Module<'static>,
    vgui2: Module<'static>,
}

impl Modules {
    fn new() -> Result<Self, ModuleError<'static>> {
        Ok(Self {
            _hw: Module::from("hw.dll")?,
            vgui2: Module::from("vgui2.dll")?,
        })
    }
}

static mut OLD_PAINT_TRAVERSE: usize = 0;
extern "fastcall" fn my_paint_traverse(this: usize, edx: usize, panel: usize, force_repaint: bool, allow_force: bool) {
    type PaintTraverseFn = extern "fastcall" fn(usize, usize, usize, bool, bool);
    let original: PaintTraverseFn = unsafe { mem::transmute(OLD_PAINT_TRAVERSE) };
    original(this, edx, panel, force_repaint, allow_force);
}

#[derive(Debug)]
pub enum Error<'a> {
    Module(ModuleError<'a>),
    NullVTable(&'a str),
}

impl<'a> From<ModuleError<'a>> for Error<'a> {
    fn from(e: ModuleError) -> Error {
        Error::Module(e)
    }
}

fn hook_and_idle(modules: &Modules) -> Result<(), Error> {
    const PANEL_INTERFACE: &str = "VGUI_Panel007";

    #[repr(usize)]
    enum PanelVtableIndex {
        PaintTraverse = 41,
        Max = 60
    }

    let panel = modules.vgui2.create_interface::<Panel>(PANEL_INTERFACE)?;
    info!("panel = {:#x?}", panel as *const _);

    if panel.vtable.is_null() {
        return Err(Error::NullVTable(PANEL_INTERFACE));
    }

    let mut vtable_copy = [0; PanelVtableIndex::Max as _];

    // Important: vtable_copy must outlive vtable_copy.as_mut_ptr().
    // Otherwise the pointer will dangle.
    unsafe { panel.vtable.copy_to_nonoverlapping(vtable_copy.as_mut_ptr(), PanelVtableIndex::Max as _); }

    vtable_copy[PanelVtableIndex::PaintTraverse as usize] = my_paint_traverse as usize;

    let patch = Patch::new(unsafe { panel.vtable.add(41) }, my_paint_traverse as usize).unwrap();
    unsafe { OLD_PAINT_TRAVERSE = patch.old_value; }

    let _hook = Hook {
        _panel: panel,
        _paint_traverse_hook: patch
    };

    idle();
    Ok(())

    // `_hook` is automatically dropped here.
    // Any cleanup logic for Hook should be specified in its implementation of Drop.
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