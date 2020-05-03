#[macro_export]
macro_rules! win_call {
    ($function:expr, $($arg:tt)*) => {{
        use winapi::um::errhandlingapi::{GetLastError, SetLastError};

        const SUCCESS: u32 = 0;

        unsafe fn you_must_wrap_the_macro_in_unsafe() {}
        you_must_wrap_the_macro_in_unsafe();

        // Set this thread's last-error value to a known success state so that
        // we can later query the error-code after a winapi call to determine
        // whether failure occurred.
        SetLastError(SUCCESS);

        let ret = ($function) ($($arg)*);

        let error_code = GetLastError();

        if error_code == SUCCESS {
            Ok(ret)
        } else {
            Err(error_code)
        }
    }};

    ($function:expr) => {{
        win_call!($function,)
    }}
}

#[macro_export]
macro_rules! wide_format {
    ($format:literal, $($arg:tt)*) => {{
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let mut widened: Vec<u16> = OsStr::new(&format!($format, $($arg)*))
            .encode_wide()
            .map(|byte| if byte == 0 {
                const REPLACEMENT_CHARACTER: u16 = 0xFFFD;
                REPLACEMENT_CHARACTER
            } else {
                byte
            })
            .collect();

        let needs_null_terminator = widened
            .last()
            .map_or(true, |last| *last != 0);

        if needs_null_terminator {
            widened.push(0);
        }

        widened
    }}
}
