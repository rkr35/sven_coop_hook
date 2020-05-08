use crate::memory::Patch;
use crate::module::{self, Module};
use crate::vgui2;

use std::mem::{self, ManuallyDrop};

use log::{error, info};
use thiserror::Error;
use wchar::wch_c as w;

// BEGIN MUTABLE GLOBAL STATE
use crate::hook::SURFACE;
static mut OLD_PAINT_TRAVERSE: usize = 0;
// END MUTABLE GLOBAL STATE

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("{0}")]
    Module(module::Error<'a>),

    #[error("interface \"{0}\" has a null vtable")]
    NullVtable(&'a str),

    #[error("tried to patch a null pointer for {0}")]
    NullPatch(&'a str),
}

impl<'a> From<module::Error<'a>> for Error<'a> {
    fn from(e: module::Error) -> Error {
        Error::Module(e)
    }
}

pub struct Hook {
    modified_vtable: ManuallyDrop<Vec<usize>>,
    vtable_patch: Patch<*mut usize>,
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

        info!("panel = {:#x?}", panel as *const _);
    
        if panel.vtable.is_null() {
            return Err(Error::NullVtable(vgui2::panel::INTERFACE));
        }
    
        let mut modified_vtable = {
            use vgui2::panel::Vtable;
    
            // Create storage for our vtable copy.
            let mut vtable = vec![0; Vtable::NumEntries as usize];
            
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
    
            ManuallyDrop::new(vtable)
        };
    
        // Replace Panel vtable with our modified vtable.
        let vtable_patch = unsafe {
            Patch::new(
                &mut panel.vtable,
                modified_vtable.as_mut_ptr()
            )
        }.ok_or(Error::NullPatch("panel vtable"))?;

        Ok(Hook {
            modified_vtable,
            vtable_patch,
        })
    }
}

extern "fastcall" fn my_paint_traverse(this: &vgui2::Panel, edx: usize, panel: &vgui2::Panel, force_repaint: bool, allow_force: bool) {
    let original: extern "fastcall" fn(&vgui2::Panel, usize, &vgui2::Panel, bool, bool) = unsafe { mem::transmute(OLD_PAINT_TRAVERSE) };
    original(this, edx, panel, force_repaint, allow_force);

    if let Some(name) = this.get_name(panel) {
        let name = name.to_bytes();
        
        if name == b"StaticPanel" {

        } else if name == b"BasePanel" {
            let surface = unsafe { &*SURFACE };
            surface.set_text_color(0, 255, 0, 255);
            surface.set_text_pos(3, 7);
            let s = w!("The quick brown fox jumps over the lazy dog.");
            surface.print_text(s);
        } 
    }
}