mod color;
mod preview;
mod win;

use color::ColorFormat;

use std::ptr::null_mut;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

use winapi::shared::windef::POINT;
use winapi::um::winuser::*;

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

    thread::spawn(|| unsafe {
        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(win::low_mouse_proc), null_mut(), 0);

        let mut msg = MSG::default();
        GetMessageW(&mut msg, null_mut(), 0, 0);

        let mut p = POINT::default();
        GetCursorPos(&mut p);

        let dc = GetDC(null_mut());
        let color = win::color_at(dc, p.x, p.y);
        ReleaseDC(null_mut(), dc);

        println!("{:x}", color);

        UnhookWindowsHookEx(hook);
    });

    thread::spawn(move || {
        // TODO: Handle me gently
        let preview = preview::create_window();

        let delay = Duration::from_secs(1/30);
        let dc = unsafe { GetDC(null_mut()) };
        let mut p = POINT::default();

        loop {
            unsafe { GetCursorPos(&mut p); }
            let color = win::color_at(dc, p.x, p.y);
            unsafe { preview::set_color(color); }

            thread::sleep(delay);
        }
        // unsafe { ReleaseDC(null_mut(), dc) };
    });

    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    };

}
