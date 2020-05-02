use std::ptr;
use wchar::wch_c as w;
use winapi::{
    um::{
        consoleapi::AllocConsole,
        libloaderapi::{DisableThreadLibraryCalls, FreeLibraryAndExitThread},
        processthreadsapi::CreateThread,
        wincon::FreeConsole,
        winnt::DLL_PROCESS_ATTACH,
        winuser::{MB_OK, MessageBoxW},
    },

    shared::{
        minwindef::{BOOL, DWORD, FALSE, HINSTANCE, LPVOID, TRUE},
    }
};

mod macros;

fn msg_box(text: &[u16], caption: &[u16]) {
    unsafe { MessageBoxW(ptr::null_mut(), text.as_ptr(), caption.as_ptr(), MB_OK); }
}

extern "system" fn on_attach(dll: LPVOID) -> DWORD {
    msg_box(w!("Okay."), w!("on_attach"));

    unsafe { AllocConsole() };

    unsafe {
        FreeConsole();
        FreeLibraryAndExitThread(dll.cast(), 0);
    }

    0
}

#[no_mangle]
#[allow(non_snake_case)]
extern "system" fn DllMain(dll: HINSTANCE, reason: DWORD, _: LPVOID) -> BOOL {
    if reason != DLL_PROCESS_ATTACH {
        return TRUE;
    }

    if let Err(error_code) = unsafe { win_call!(DisableThreadLibraryCalls, dll) } {
        let text = wide_format!("DisableThreadLibraryCalls failed. GetLastError = {:#x}", error_code);
        msg_box(&text, w!("DllMain error"));
    } else if let Err(error_code) = unsafe { win_call!(CreateThread, ptr::null_mut(), 0, Some(on_attach), dll.cast(), 0, ptr::null_mut()) } {
        let text = wide_format!("CreateThread failed. GetLastError = {:#x}", error_code);
        msg_box(&text, w!("DllMain error"));
    }

    TRUE
}