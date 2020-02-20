use crate::color::Color;

use std::ffi::OsStr;
use std::ptr::null_mut;

use winapi::um::wingdi::GetPixel;
use winapi::um::winnt::HRESULT;
use winapi::um::winuser::*;

use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::{HDC, COLORREF};
use winapi::shared::winerror::S_OK;

use wio::wide::ToWide;

// TODO: Try trait should be able to replace this macro in future
// https://doc.rust-lang.org/std/ops/trait.Try.html
pub fn hr(code: HRESULT) -> Result<(), HRESULT> {
    match code {
        S_OK => Ok(()),
        _ => Err(code),
    }
}

macro_rules! void {
    ($p:expr, $t:ty) => {
        &mut $p as *mut *mut $t as _
    };
}

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
    let mut text_wide = OsStr::new(text).to_wide_null();

    unsafe {
        OpenClipboard(null_mut());
        EmptyClipboard();
        SetClipboardData(CF_UNICODETEXT, text_wide.as_mut_ptr() as _);
        CloseClipboard();
    }
}
