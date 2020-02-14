use crate::color::Color;

use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

use winapi::um::wingdi::GetPixel;
use winapi::um::winuser::*;

use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::{HDC, COLORREF};

// Synchronous. Must be called in a separate thread.
pub unsafe extern "system" fn low_mouse_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(null_mut(), code, w_param, l_param);
    }

    match w_param as u32 {
        WM_LBUTTONDOWN => {
            PostQuitMessage(0);
            return -1;
        }
        _ => {}
    }

    return CallNextHookEx(null_mut(), code, w_param, l_param);
}

pub fn color_at(hdc: HDC, x: i32, y: i32) -> Color {
    let color: COLORREF = unsafe { GetPixel(hdc, x, y) };

    Color(color.swap_bytes() >> 8)
}

pub fn copy_to_clipboard(text: &str) {
    let mut text_wide = to_wide(text);

    unsafe {
        OpenClipboard(null_mut());
        EmptyClipboard();
        SetClipboardData(CF_UNICODETEXT, text_wide.as_mut_ptr() as _);
        CloseClipboard();
    }
}

pub fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}
