use crate::color::to_rgb;
use crate::win;

use std::io;
use std::ptr::{null, null_mut};

use failure::*;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::*;

use winapi::Interface;
use winapi::shared::winerror::*;
use winapi::shared::windef::{HDC, POINT};
use winapi::um::winuser::{PAINTSTRUCT, RegisterClassW};
use winapi::um::winuser::WNDCLASSW;
use winapi::um::winuser::*;
use winapi::um::d2d1::*;
use winapi::um::d2d1::ID2D1Factory;

static mut COLOR: u32 = 0;

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> LRESULT {
    match msg {
        WM_PAINT => {
            let ps: *mut PAINTSTRUCT = null_mut();
            BeginPaint(hwnd, ps);

            EndPaint(hwnd, ps);
            return 0;
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            return 0;
        }
        _ => {}
    }

    return DefWindowProcW(hwnd, msg, wparam, lparam);
}

pub fn create_window() -> Result<HWND, Error> {
    let class_name = win::to_wide("wcolor");
    let title = win::to_wide("WColor");

    unsafe {
        let hinstance = GetModuleHandleW(null_mut());

        let wnd_class = WNDCLASSW {
            lpszClassName: class_name.as_ptr(),
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance,
            ..WNDCLASSW::default()
        };

        RegisterClassW(&wnd_class);

        let mut p = POINT::default();
        GetCursorPos(&mut p);

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUPWINDOW | WS_VISIBLE,
            p.x + 20,
            p.y + 20,
            128,
            128,
            null_mut(),
            null_mut(),
            hinstance,
            null_mut(),
        );

        if hwnd.is_null() {
            return Err(io::Error::last_os_error().into());
        }

        let mut d2d1_factory: *mut ID2D1Factory = null_mut();

        let result = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, &mut ID2D1Factory::uuidof(), null(), &mut d2d1_factory as *mut *mut ID2D1Factory as *mut *mut winapi::ctypes::c_void);
        if result != S_OK {
            panic!("Couldn't create D2D1 factory, error code: {}", result);
        }

        // (*d2d1_factory).CreateHwndRenderTarget(null(), null(), null());

        Ok(hwnd)
    }
}

pub unsafe fn set_color(new_color: u32) {
    if new_color == COLOR {
        return;
    }

    COLOR = new_color;
}
