use crate::memory::{self, Patch};
use crate::module::{self, Module};
use crate::game::vgui2;

use std::mem::{self, ManuallyDrop};

use log::{error, info};
use thiserror::Error;

// BEGIN MUTABLE GLOBAL STATE
static mut OLD_PAINT_TRAVERSE: usize = 0;
// END MUTABLE GLOBAL STATE

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("{0}")]
    Module(module::Error<'a>),

    #[error("interface \"{0}\" has a null vtable")]
    NullVtable(&'a str),

    #[error("patch error: {0}")]
    Patch(#[from] memory::Error),
}

impl<'a> From<module::Error<'a>> for Error<'a> {
    fn from(e: module::Error) -> Error {
        Error::Module(e)
    }
}

pub struct Hook {
    vtable_patch: Patch<*mut [usize; vgui2::panel::NUM_VTABLE_ENTRIES]>,
    modified_vtable: ManuallyDrop<Box<[usize; vgui2::panel::NUM_VTABLE_ENTRIES]>>,
}

impl Drop for Hook {
    fn drop(&mut self) {
        unsafe {
            self.vtable_patch.restore();
            
            // Only drop the modified panel vtable after we restore the original vtable;
            // otherwise, panel will access deallocated vtable entries.
            ManuallyDrop::drop(&mut self.modified_vtable);
        }

        info!("Panel hook dropped.");
    }
}

impl Hook {
    pub fn new(vgui2: &Module) -> Result<Hook, Error<'static>> {
        let panel = vgui2.create_interface::<vgui2::Panel>(vgui2::panel::INTERFACE)?;

        info!("panel = {:#x?}", panel);
    
        if unsafe { (*panel).vtable.is_null() } {
            return Err(Error::NullVtable(vgui2::panel::INTERFACE));
        }
    
        let mut modified_vtable = {
            unsafe { 
                // Copy the original Panel vtable to our vtable.
                let mut vtable = Box::new(*(*panel).vtable);

                // Hook PaintTraverse and save the original.
                OLD_PAINT_TRAVERSE = mem::replace(
                    &mut vtable[41],
                    my_paint_traverse as usize,
                );

                ManuallyDrop::new(vtable)
            }
        };
    
        // Replace Panel vtable with our modified vtable.
        let vtable_patch = unsafe {
            Patch::new(
                &mut (*panel).vtable,
                modified_vtable.as_mut()
            )
        }?;

        Ok(Hook {
            modified_vtable,
            vtable_patch,
        })
    }
}

unsafe extern "fastcall" fn my_paint_traverse(this: *const vgui2::Panel, edx: usize, panel: *const vgui2::Panel, force_repaint: bool, allow_force: bool) {
    type PaintTraverse = extern "fastcall" fn(*const vgui2::Panel, usize, *const vgui2::Panel, bool, bool);
    let original: PaintTraverse = mem::transmute(OLD_PAINT_TRAVERSE);
    original(this, edx, panel, force_repaint, allow_force);

    if let Some(name) = (*this).get_name(panel) {
        let _ = name.to_bytes();
    }
}