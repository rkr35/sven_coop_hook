use crate::idle;
use crate::memory::Patch;
use crate::module::{Error as ModuleError, Module};
use crate::vgui::Panel;
use std::mem;

use log::{error, info};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("module error: {0}")]
    Module(ModuleError<'a>),

    #[error("interface \"{0}\" has a null vtable")]
    NullVTable(&'a str),
}

impl<'a> From<ModuleError<'a>> for Error<'a> {
    fn from(e: ModuleError) -> Error {
        Error::Module(e)
    }
}

struct Hook {
    panel_vtable: Patch<*mut usize>,
}

impl Drop for Hook {
    fn drop(&mut self) {
        unsafe { self.panel_vtable.restore(); }
        info!("Hook dropped.");
    }
}

struct Modules {
    _hw: Module,
    vgui2: Module,
}

impl Modules {
    fn new() -> Result<Modules, ModuleError<'static>> {
        Ok(Modules {
            _hw: Module::from("hw.dll")?,
            vgui2: Module::from("vgui2.dll")?,
        })
    }
}

static mut OLD_PAINT_TRAVERSE: usize = 0;
extern "fastcall" fn my_paint_traverse(this: usize, edx: usize, panel: usize, force_repaint: bool, allow_force: bool) {
    let original: extern "fastcall" fn(usize, usize, usize, bool, bool) = unsafe { mem::transmute(OLD_PAINT_TRAVERSE) };
    original(this, edx, panel, force_repaint, allow_force);
}

fn hook_and_idle(modules: Modules) -> Result<(), Error<'static>> {
    const PANEL_INTERFACE: &str = "VGUI_Panel007";

    #[repr(usize)]
    enum PanelVtable {
        PaintTraverse = 41,
        Max = 60
    }

    let panel = modules.vgui2.create_interface::<Panel>(PANEL_INTERFACE)?;

    info!("panel = {:#x?}", panel as *const _);

    if panel.vtable.is_null() {
        return Err(Error::NullVTable(PANEL_INTERFACE));
    }

    let mut modified_vtable = {
        // Create storage for our vtable copy.
        let mut vtable = [0; PanelVtable::Max as _];
        
        // Copy the original Panel vtable to our vtable.
        unsafe { panel.vtable.copy_to_nonoverlapping(vtable.as_mut_ptr(), PanelVtable::Max as _); }

        // Hook PaintTraverse.
        vtable[PanelVtable::PaintTraverse as usize] = my_paint_traverse as usize;

        vtable
    };

    // Set Panel's vtable to our vtable which contains the PaintTraverse hook.
    // SAFETY: You must ensure that `modified_vtable` outlives `modified_vtable.as_mut_ptr()`.
    // Otherwise this mutable pointer will be a dangling reference. The easiest way to satisfy
    // this safety requirement is to not move `modified_vtable` while this patch is alive.
    let patch = unsafe {
        // TODO: Gracefully handle `.unwrap()`
        let p = Patch::new(&mut panel.vtable, modified_vtable.as_mut_ptr()).unwrap();
        
        // Save the address of the original PaintTraverse function, so we can call it in our hook.
        // TODO: Gracefully handle null vtable entry.
        OLD_PAINT_TRAVERSE = *p.old_value.add(PanelVtable::PaintTraverse as usize);

        p
    };

    {
        let _hook = Hook {
            panel_vtable: patch,
        };

        idle();

        // `_hook` dropped here.
        // Any cleanup logic, e.g., restoring patches, should be done in Hook's implementation of Drop.
    }

    Ok(())
}

pub fn run() -> Result<(), Error<'static>> {
    hook_and_idle(Modules::new()?)?;
    Ok(())
}