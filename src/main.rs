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
    #[structopt(short = "f", long = "format", possible_values = &["HEX", "hex", "RGB"], default_value = "HEX")]
    format: String,

    #[structopt(short = "n", long = "no-preview")]
    no_preview: bool,

    #[structopt(short = "c", long = "clipboard")]
    clipboard: bool,
}

fn main() {
    let args: Args = Args::from_args();

    //TODO: Probably not the right way to it
    let format = args.format;
    let clipboard = args.clipboard;

    let thread_id = unsafe { GetCurrentThreadId() };

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

        let output = match format.as_str() {
            "HEX" => { format!("#{:X}", color.0) },
            "hex" => { format!("#{:x}", color.0) },
            "RGB" => {
                let (r, g, b) = color.to_rgb();
                format!("{}, {}, {}", r, g, b)
            }
            _ => { unreachable!() }
        };

        if clipboard {
            win::copy_to_clipboard(&output);
        }

        println!("{}", output);

        PostThreadMessageW(thread_id, WM_QUIT, 0,0);
    });

    if !args.no_preview {
        // TODO: Handle me gently
        let mut preview = preview::create_preview().unwrap();

        unsafe {
            SetWindowLongPtrW(preview.hwnd, 0 , &mut preview as *mut Preview as _);
        }

        thread::spawn(move || {
            let delay = Duration::from_millis(5);

            let dc = unsafe { GetDC(null_mut()) };
            let mut pt = POINT::default();

            loop {
                unsafe {
                    GetCursorPos(&mut pt);
                    SetWindowPos(preview.hwnd, null_mut(), pt.x + 10, pt.y + 15, 0, 0, SWP_NOSIZE);
                }

                let color = win::color_at(dc, pt.x, pt.y);
                if preview.set_color(color) {
                    preview.draw();
                }

                thread::sleep(delay);
            }
        });
    }

    let mut msg = MSG::default();

    unsafe {
        while GetMessageW(&mut msg, null_mut(), 0, 0) != 0 {
            DispatchMessageW(&msg);
        }
    }

}
