#[macro_use]
mod win;
mod color;
mod preview;

use color::ColorFormat;
use preview::Preview;

use std::ptr::null_mut;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

use winapi::shared::windef::POINT;
use winapi::um::winuser::*;
use winapi::um::processthreadsapi::GetCurrentThreadId;

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "f", long, possible_values = &["HEX", "rgb"], default_value = "HEX")]
    format: ColorFormat,

    #[structopt(short = "n", long = "no-preview")]
    no_preview: bool,

    #[structopt(short = "c", long)]
    clipboard: bool,
}


fn main() {
    let args = Args::from_args();

    let thread_id = unsafe { GetCurrentThreadId() };

    // TODO: Handle me gently
    let mut preview = preview::create_preview().unwrap();

    unsafe {
        SetWindowLongPtrW(preview.hwnd, 0 , &mut preview as *mut Preview as _);
    }

    thread::spawn(move || unsafe {
        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(win::low_mouse_proc), null_mut(), 0);

        let mut msg = MSG::default();
        GetMessageW(&mut msg, null_mut(), 0, 0);

        UnhookWindowsHookEx(hook);

        let mut p = POINT::default();
        GetCursorPos(&mut p);

        let dc = GetDC(null_mut());
        let color = win::color_at(dc, p.x, p.y);
        ReleaseDC(null_mut(), dc);

        println!("#{:X}", color.0);

        PostThreadMessageW(thread_id, WM_QUIT, 0,0);
    });

    let delay = Duration::from_secs(1/30);
    let mut msg = MSG::default();
    let mut pt: POINT = POINT::default();

    unsafe {
        let dc = GetDC(null_mut());

        loop {
            if PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                DispatchMessageW(&msg);
                if msg.message == WM_QUIT {
                    break
                }
            }

            GetCursorPos(&mut pt);
            let color = win::color_at(dc, pt.x, pt.y);
            preview.set_color(color);
            preview.render();
            // MoveWindow(preview.hwnd, pt.x + 20, pt.y + 20, 32, 32, 1);
            SetWindowPos(preview.hwnd, null_mut(), pt.x + 20, pt.y + 20, 0, 0, SWP_NOSIZE | SWP_NOREDRAW | SWP_ASYNCWINDOWPOS);

            // thread::sleep(delay);
        }

        ReleaseDC(null_mut(), dc);
        preview.release();
    }

    // unsafe {
        // while GetMessageW(&mut msg, null_mut(), 0, 0) != 0 {
            // DispatchMessageW(&msg);
        // }

        // preview.release();
    // }

    // let delay = Duration::from_secs(1/30);
    // let dc = unsafe { GetDC(null_mut()) };
    // let mut p = POINT::default();

    // loop {
    // unsafe { GetCursorPos(&mut p); }
    // let color = win::color_at(dc, p.x, p.y);
    // unsafe { preview::set_color(color); }

    // thread::sleep(delay);
    // }
    // unsafe { ReleaseDC(null_mut(), dc) };

}
