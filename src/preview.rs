use crate::color::Color;
use crate::win;

use std::io::Error;
use std::ptr::{null, null_mut};

use winapi::Interface;

use winapi::um::dcommon::{D2D1_PIXEL_FORMAT, D2D1_ALPHA_MODE_PREMULTIPLIED};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;
use winapi::um::d2d1::*;
// use winapi::um::d2d1_2::*;

use winapi::shared::winerror::{HRESULT, S_OK, E_POINTER};
use winapi::shared::windef::{HWND, RECT, POINT};
use winapi::shared::minwindef::LRESULT;
use winapi::shared::dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM;
use winapi::shared::d3d9types::D3DCOLORVALUE;


pub struct Preview {
    pub hwnd: HWND,
    render_target: *mut ID2D1HwndRenderTarget,
    brush: *mut ID2D1SolidColorBrush,
    ellipse: D2D1_ELLIPSE,
}

unsafe impl Send for Preview {}

impl Preview {
    pub fn set_color(&mut self, color: Color) -> bool {
        let brush_color = unsafe { (*self.brush).GetColor() };
        let d3d_color = D3DCOLORVALUE::from(color);

        if brush_color.r == d3d_color.r || brush_color.g == d3d_color.g || brush_color.b == d3d_color.b {
            return false;
        }

        unsafe { (*self.brush).SetColor(&d3d_color as *const _) };

        return true;
    }

    pub fn draw(&mut self) -> HRESULT {
        if self.render_target.is_null() || self.brush.is_null() {
            return E_POINTER;
        }

        unsafe {
            let target = &*self.render_target;
            let color = (*self.brush).GetColor();

            target.BeginDraw();

            target.FillEllipse(&self.ellipse as *const _, self.brush as *mut ID2D1Brush);

            let negative = D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
            (*self.brush).SetColor(&negative as *const _);

            target.DrawEllipse(&self.ellipse as *const _, self.brush as *mut ID2D1Brush, 3.0, null_mut());

            (*self.brush).SetColor(&color as *const _);

            target.EndDraw(null_mut(), null_mut())
        }
    }

    pub unsafe fn release(&mut self) {
        if !self.render_target.is_null() {
            (*self.brush).Release();
            (*self.render_target).Release();

            self.brush = null_mut();
            self.render_target = null_mut();
        }
    }
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> LRESULT {
    let preview: &mut Preview = &mut *(GetWindowLongPtrW(hwnd, 0) as *mut Preview);

    match msg {
        WM_PAINT => {
            let ps: *mut PAINTSTRUCT = null_mut();
            BeginPaint(hwnd, ps);

            preview.draw();

            EndPaint(hwnd, ps);
            return 0;
        }
        WM_DESTROY => {
            preview.release();
            PostQuitMessage(0);
            return 0;
        }
        _ => {}
    }

    return DefWindowProcW(hwnd, msg, wparam, lparam);
}

pub fn create_preview() -> Result<Preview, Error> {
    let class_name = win::to_wide("wcolor");
    let title = win::to_wide("WColor");

    let hwnd: HWND;

    unsafe {
        let hinstance = GetModuleHandleW(null_mut());

        let wnd_class = WNDCLASSW {
            lpszClassName: class_name.as_ptr(),
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance,
            cbWndExtra: 8,
            ..WNDCLASSW::default()
        };

        RegisterClassW(&wnd_class);

        let mut p = POINT::default();
        GetCursorPos(&mut p);

        hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_TOPMOST,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP | WS_VISIBLE,
            p.x + 20,
            p.y + 20,
            32,
            32,
            null_mut(),
            null_mut(),
            hinstance,
            null_mut(),
        );
    }

    if hwnd.is_null() {
        return Err(Error::last_os_error().into());
    }

    unsafe { SetLayeredWindowAttributes(hwnd, 0, 255, ULW_COLORKEY | LWA_ALPHA); }

    // TODO: Handle me gently
    let factory = create_factory().unwrap();
    let render_target = create_render_target(&hwnd, unsafe { &*factory }).unwrap();

    unsafe { (*factory).Release(); }

    let mut brush: *mut ID2D1SolidColorBrush = null_mut();

    let color_transparent = D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    unsafe {
        (*render_target).CreateSolidColorBrush(&color_transparent, null_mut(), &mut brush as *mut *mut ID2D1SolidColorBrush as _);
    }

    let radius = 10.0;
    let center = D2D1_POINT_2F { x: radius, y: radius };
    let ellipse = D2D1_ELLIPSE { point: center, radiusX: radius, radiusY: radius };

    Ok(Preview { hwnd, render_target, brush, ellipse })
}

fn create_factory() -> Result<*mut ID2D1Factory, HRESULT> {
    let mut factory: *mut ID2D1Factory = null_mut();

    let error = unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, &mut ID2D1Factory::uuidof(), null(), &mut factory as *mut *mut ID2D1Factory as *mut *mut winapi::ctypes::c_void) };

    if error != S_OK {
        return Err(error);
    }

    Ok(factory)
}

fn create_render_target(hwnd: &HWND, factory: &ID2D1Factory) -> Result<*mut ID2D1HwndRenderTarget, HRESULT> {
    let pixel_format = D2D1_PIXEL_FORMAT {
        format: DXGI_FORMAT_B8G8R8A8_UNORM,
        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED
    };

    let properties = D2D1_RENDER_TARGET_PROPERTIES {
        _type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
        pixelFormat: pixel_format,
        ..D2D1_RENDER_TARGET_PROPERTIES::default()
    };

    let mut rc: RECT = RECT::default();
    unsafe { GetClientRect(*hwnd, &mut rc) };

    let hwnd_properties = D2D1_HWND_RENDER_TARGET_PROPERTIES {
        hwnd: *hwnd,
        pixelSize: D2D1_SIZE_U { width: (rc.right - rc.left) as u32, height: (rc.bottom - rc.top) as u32 },
        presentOptions: D2D1_PRESENT_OPTIONS_IMMEDIATELY
    };

    let mut render_target: *mut ID2D1HwndRenderTarget = null_mut();
    unsafe {
        let error = factory.CreateHwndRenderTarget(&properties, &hwnd_properties, &mut render_target);

        if error != S_OK {
            return Err(error);
        }

        (*render_target).SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
    }

    Ok(render_target)
}
