use std::io::{self, Read};
use std::panic;
use std::ptr;

use log::info;
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};
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
        minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE},
    }
};

mod macros;
mod hook;
use hook::Hook;

fn msg_box(text: &[u16], caption: &[u16]) {
    unsafe { MessageBoxW(ptr::null_mut(), text.as_ptr(), caption.as_ptr(), MB_OK); }
}

fn idle() {
    info!("Idling. Press enter to continue.");
    let mut sentinel = [0; 2];
    let _ = io::stdin().read_exact(&mut sentinel);
}

fn hook() {
    let _hook = Hook;
    idle();
}

extern "system" fn on_attach(dll: LPVOID) -> DWORD {
    msg_box(w!("Okay."), w!("on_attach"));

    let result = panic::catch_unwind(|| {
        unsafe { AllocConsole() };
        println!("Allocated console.");

        if let Err(e) = TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed) {
            eprintln!("Failed to initialize logger: {}", e);
        } else {
            info!("Initialized logger.");
            hook();
        }

        idle();
    });

    if result.is_err() {
        let text = w!("on_attach() caught a panic. The state of the hook is unknown. The hook will now detach.");
        msg_box(text, w!("Detaching because of a panic."));
    }

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