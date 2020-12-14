use crate::color::Color;
use crate::win::hr;

use std::ffi::OsStr;
use std::io::Error;
use std::ptr::{null, null_mut};

use wio::com::ComPtr;
use wio::wide::ToWide;
use winapi::Interface;

use winapi::um::dcommon::{D2D1_PIXEL_FORMAT, D2D1_ALPHA_MODE_PREMULTIPLIED};
use winapi::um::dcomp::{DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget, IDCompositionVisual};
use winapi::um::d3dcommon::D3D_DRIVER_TYPE_HARDWARE;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;
use winapi::um::d2d1::*;
use winapi::um::d2d1_1::{ID2D1Bitmap1, D2D1_BITMAP_PROPERTIES1, D2D1_DEVICE_CONTEXT_OPTIONS_NONE, D2D1_BITMAP_OPTIONS_TARGET, D2D1_BITMAP_OPTIONS_CANNOT_DRAW};
use winapi::um::d2d1_2::{ID2D1Device1, ID2D1Factory2, ID2D1DeviceContext1, ID2D1GeometryRealization};
use winapi::um::d3d11::{D3D11CreateDevice, ID3D11Device, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION};

use winapi::shared::winerror::HRESULT;
use winapi::shared::windef::{HWND, RECT, POINT};
use winapi::shared::minwindef::LRESULT;
use winapi::shared::dxgi::*;
use winapi::shared::dxgi1_2::*;
use winapi::shared::dxgi1_3::{CreateDXGIFactory2, DXGI_CREATE_FACTORY_DEBUG};
use winapi::shared::dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM;
use winapi::shared::d3d9types::D3DCOLORVALUE;
use winapi::shared::dxgitype::{DXGI_SAMPLE_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT};

pub struct Preview {
    pub hwnd:    HWND,
    context:     ComPtr<ID2D1DeviceContext1>,
    swap_chain:  ComPtr<IDXGISwapChain1>,
    _target:     ComPtr<IDCompositionTarget>,
    brush:       ComPtr<ID2D1SolidColorBrush>,
    ellipse:     ComPtr<ID2D1GeometryRealization>,
    stroke:      ComPtr<ID2D1GeometryRealization>,
}

unsafe impl Send for Preview {}

impl Preview {
    pub fn set_color(&mut self, color: Color) -> bool {
        let brush_color = unsafe { (*self.brush).GetColor() };
        let d3d_color = D3DCOLORVALUE::from(color);

        if brush_color.r == d3d_color.r && brush_color.g == d3d_color.g && brush_color.b == d3d_color.b {
            return false;
        }

        unsafe { (*self.brush).SetColor(&d3d_color as *const _) };

        return true;
    }

    pub fn draw(&mut self) -> HRESULT {
        unsafe {
            let context = &*self.context;
            let brush = &*self.brush;

            context.BeginDraw();

            context.Clear(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 });

            context.DrawGeometryRealization(self.ellipse.as_raw(), self.brush.as_raw() as _);

            let color = brush.GetColor();
            let negative = D2D1_COLOR_F { r: 0.1, g: 0.1, b: 0.1, a: 1.0 };
            brush.SetColor(&negative as *const _);

            context.DrawGeometryRealization(self.stroke.as_raw(), self.brush.as_raw() as _);

            brush.SetColor(&color as *const _);

            context.EndDraw(null_mut(), null_mut());

            (*self.swap_chain).Present(1, 0)
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
            PostQuitMessage(0);
            return 0;
        }
        _ => {}
    }

    return DefWindowProcW(hwnd, msg, wparam, lparam);
}

pub fn create_preview() -> Result<Preview, Error> {
    let name = OsStr::new("wcolor").to_wide_null();

    let hwnd: HWND;

    unsafe {
        let hinstance = GetModuleHandleW(null_mut());

        let wnd_class = WNDCLASSW {
            lpszClassName: name.as_ptr(),
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance,
            cbWndExtra: 8,
            ..WNDCLASSW::default()
        };

        RegisterClassW(&wnd_class);

        let mut p = POINT::default();
        GetCursorPos(&mut p);

        hwnd = CreateWindowExW(
            WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            name.as_ptr(),
            name.as_ptr(),
            WS_POPUP | WS_VISIBLE,
            p.x + 20,
            p.y + 20,
            24,
            24,
            null_mut(),
            null_mut(),
            hinstance,
            null_mut(),
        );

        if hwnd.is_null() {
            return Err(Error::last_os_error().into());
        }

        // TODO: Handle me gently
        let device = create_dxgi_device().unwrap();
        let factory = create_factory().unwrap();
        let context = create_device_context(&device, &factory).unwrap();
        let swap_chain = create_swap_chain(hwnd, &device).unwrap();
        create_bitmap(&context, &swap_chain).unwrap();
        let _target = create_composition(hwnd, &device, &swap_chain).unwrap();

        let mut brush: *mut ID2D1SolidColorBrush = null_mut();
        (*context).CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, null_mut(), void!(brush, ID2D1SolidColorBrush));

        let radius = 10.0;
        let center = D2D1_POINT_2F { x: radius + 2.0, y: radius + 2.0 };
        let ellipse = D2D1_ELLIPSE { point: center, radiusX: radius, radiusY: radius };

        let mut ellipse_geometry: *mut ID2D1EllipseGeometry = null_mut();
        (*factory).CreateEllipseGeometry(&ellipse, &mut ellipse_geometry);

        let mut ellipse_realization: *mut ID2D1GeometryRealization = null_mut();
        (*context).CreateFilledGeometryRealization(ellipse_geometry as _, D2D1_DEFAULT_FLATTENING_TOLERANCE, &mut ellipse_realization);

        let mut stroke_realization: *mut ID2D1GeometryRealization = null_mut();
        (*context).CreateStrokedGeometryRealization(ellipse_geometry as _, D2D1_DEFAULT_FLATTENING_TOLERANCE, 1.3, null_mut(), &mut stroke_realization);

        (*ellipse_geometry).Release();

        Ok(Preview {
            hwnd,
            context,
            swap_chain,
            _target,
            brush: ComPtr::from_raw(brush),
            ellipse: ComPtr::from_raw(ellipse_realization),
            stroke: ComPtr::from_raw(stroke_realization)
        })
    }
}

fn create_dxgi_device() -> Result<ComPtr<IDXGIDevice>, HRESULT> {
    unsafe {
        let mut device: *mut ID3D11Device = null_mut();
        hr(D3D11CreateDevice(null_mut(), D3D_DRIVER_TYPE_HARDWARE, null_mut(), D3D11_CREATE_DEVICE_BGRA_SUPPORT, null(), 0, D3D11_SDK_VERSION, &mut device, null_mut(), null_mut()))?;

        let device = ComPtr::from_raw(device);
        let dxgi_device = device.cast::<IDXGIDevice>()?;

        Ok(dxgi_device)
    }
}

fn create_factory() -> Result<ComPtr<ID2D1Factory2>, HRESULT> {
    let mut d2d_factory: *mut ID2D1Factory2 = null_mut();

    let options = D2D1_FACTORY_OPTIONS {
        debugLevel: if cfg!(debug_assertions) { D2D1_DEBUG_LEVEL_INFORMATION } else { 0 }
    };

    unsafe {
        hr(D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, &ID2D1Factory2::uuidof(), &options, void!(d2d_factory, ID2D1Factory2)))?;

        Ok(ComPtr::from_raw(d2d_factory))
    }
}

fn create_device_context(dxgi_device: &ComPtr<IDXGIDevice>, factory: &ComPtr<ID2D1Factory2>) -> Result<ComPtr<ID2D1DeviceContext1>, HRESULT> {
    unsafe {
        let mut d2d_device: *mut ID2D1Device1 = null_mut();
        hr((*factory).CreateDevice(dxgi_device.as_raw(), &mut d2d_device))?;

        let mut dc: *mut ID2D1DeviceContext1 = null_mut();
        (*d2d_device).CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE, &mut dc);

        Ok(ComPtr::from_raw(dc))
    }
}

fn create_swap_chain(hwnd: HWND, device: &ComPtr<IDXGIDevice>) -> Result<ComPtr<IDXGISwapChain1>, HRESULT> {
    unsafe {
        let mut dxgi_factory: *mut IDXGIFactory2 = null_mut();
        hr(CreateDXGIFactory2(if cfg!(debug_assertions) { DXGI_CREATE_FACTORY_DEBUG } else { 0 }, &IDXGIFactory2::uuidof(), void!(dxgi_factory, IDXGIFactory2)))?;

        let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        GetClientRect(hwnd, &mut rect);

        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: (rect.right - rect.left) as u32,
            Height: (rect.bottom - rect.top) as u32,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            Stereo: 0,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0, },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
            Flags: 0,
        };

        let mut swap_chain: *mut IDXGISwapChain1 = null_mut();
        hr((*dxgi_factory).CreateSwapChainForComposition(device.as_raw() as _, &swap_chain_desc, null_mut(), &mut swap_chain))?;

        (*dxgi_factory).Release();

        Ok(ComPtr::from_raw(swap_chain))
    }
}

fn create_bitmap(dc: &ComPtr<ID2D1DeviceContext1>, swap_chain: &ComPtr<IDXGISwapChain1>) -> Result<ComPtr<ID2D1Bitmap1>, HRESULT> {
    unsafe {
        let mut dxgi_buffer: *mut IDXGISurface2 = null_mut();
        hr((*swap_chain).GetBuffer(0, &IDXGISurface2::uuidof(), void!(dxgi_buffer, IDXGISurface2)))?;

        let properties = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
            dpiX: 0.0,
            dpiY: 0.0,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
            colorContext: null_mut(),
        };

        let mut bitmap: *mut ID2D1Bitmap1 = null_mut();
        hr((*dc).CreateBitmapFromDxgiSurface(dxgi_buffer as _, &properties, &mut bitmap))?;

        (*dc).SetTarget(bitmap as _);

        Ok(ComPtr::from_raw(bitmap))
    }
}

fn create_composition(hwnd: HWND, device: &ComPtr<IDXGIDevice>, swap_chain: &ComPtr<IDXGISwapChain1>) -> Result<ComPtr<IDCompositionTarget>, HRESULT> {
    unsafe {
        let mut dcomp_device: *mut IDCompositionDevice = null_mut();
        hr(DCompositionCreateDevice(device.as_raw(), &IDCompositionDevice::uuidof(), void!(dcomp_device, IDCompositionDevice)))?;

        let mut target: *mut IDCompositionTarget = null_mut();
        hr((*dcomp_device).CreateTargetForHwnd(hwnd, 1, &mut target))?;

        let mut visual: *mut IDCompositionVisual = null_mut();
        hr((*dcomp_device).CreateVisual(&mut visual))?;

        hr((*visual).SetContent(swap_chain.as_raw() as _))?;
        hr((*target).SetRoot(visual as _))?;
        hr((*dcomp_device).Commit())?;

        (*dcomp_device).Release();
        (*visual).Release();

        Ok(ComPtr::from_raw(target))
    }
}
