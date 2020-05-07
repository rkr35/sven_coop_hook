use crate::idle;
use crate::memory::Patch;
use crate::module::{Error as ModuleError, Module};
use crate::vgui2;
use std::mem;

use log::{error, info};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("module error: {0}")]
    Module(ModuleError<'a>),

    #[error("interface \"{0}\" has a null vtable")]
    NullVtable(&'a str),

    #[error("tried to patch a null pointer for {0}")]
    NullPatch(&'a str),
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
        unsafe {
            self.panel_vtable.restore();
        }
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
extern "fastcall" fn my_paint_traverse(this: &vgui2::Panel, edx: usize, panel: usize, force_repaint: bool, allow_force: bool) {
    let original: extern "fastcall" fn(&vgui2::Panel, usize, usize, bool, bool) = unsafe { mem::transmute(OLD_PAINT_TRAVERSE) };
    original(this, edx, panel, force_repaint, allow_force);
}

fn hook_and_idle(modules: Modules) -> Result<(), Error<'static>> {
    let panel = modules.vgui2.create_interface::<vgui2::Panel>(vgui2::panel::INTERFACE)?;

    info!("panel = {:#x?}", panel as *const _);

    if panel.vtable.is_null() {
        return Err(Error::NullVtable(vgui2::panel::INTERFACE));
    }

    let mut modified_vtable = {
        use vgui2::panel::Vtable;

        // Create storage for our vtable copy.
        let mut vtable = [0; Vtable::NumEntries as usize];
        
        unsafe { 
            // Copy the original Panel vtable to our vtable.
            panel
                .vtable
                .copy_to_nonoverlapping(vtable.as_mut_ptr(), vtable.len());

            // Hook PaintTraverse and save the original.
            OLD_PAINT_TRAVERSE = mem::replace(
                &mut vtable[Vtable::PaintTraverse as usize],
                my_paint_traverse as usize,
            );
        }

        vtable
    };

    // Set Panel's vtable to our vtable which contains the PaintTraverse hook.
    // SAFETY: You must ensure that `modified_vtable` outlives `modified_vtable.as_mut_ptr()`.
    // Otherwise this mutable pointer will be a dangling reference. The easiest way to satisfy
    // this safety requirement is to not move `modified_vtable` while this patch is alive.
    let patch = unsafe {
        Patch::new(&mut panel.vtable, modified_vtable.as_mut_ptr())
            .ok_or(Error::NullPatch("panel vtable"))?
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
