#[cfg(windows)]
extern crate winapi;

use core::cmp::Ordering;
use core::mem::MaybeUninit;

use winapi::um::wingdi::{
    wglCreateContext, wglMakeCurrent, ChoosePixelFormat, SetPixelFormat, SwapBuffers, DEVMODEA,
    PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_SUPPORT_OPENGL, PFD_TYPE_RGBA, PIXELFORMATDESCRIPTOR,
};

use winapi::shared::minwindef::{HINSTANCE, LPARAM, LPVOID, LRESULT, UINT, WPARAM};

use winapi::shared::windef::{HBRUSH, HDC, HGLRC, HICON, HMENU, HWND};

use winapi::um::libloaderapi::GetModuleHandleA;

use winapi::um::winuser::{
    CreateWindowExA, DefWindowProcA, DispatchMessageA, GetDC, MessageBoxA, PeekMessageA,
    PostQuitMessage, RegisterClassA, TranslateMessage, CS_HREDRAW, CS_OWNDC, CS_VREDRAW,
    CW_USEDEFAULT, MB_ICONERROR, MSG, PM_REMOVE, WNDCLASSA, WS_MAXIMIZE, WS_OVERLAPPEDWINDOW,
    WS_POPUP, WS_VISIBLE,
};

pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        winapi::um::winuser::WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => {
            return DefWindowProcA(hwnd, msg, w_param, l_param);
        }
    }
    return 0;
}

#[cfg(feature = "logger")]
pub fn show_error(message: *const i8) {
    unsafe {
        MessageBoxA(
            0 as HWND,
            message,
            "Window::create\0".as_ptr() as *const i8,
            MB_ICONERROR,
        );
    }
}

pub fn create_window() -> (HWND, HDC) {
    unsafe {
        let h_wnd: HWND;

        #[cfg(feature = "fullscreen")]
        {
            let mut dev_mode: DEVMODEA = core::mem::zeroed();
            dev_mode.dmSize = core::mem::size_of::<DEVMODEA>() as u16;
            dev_mode.dmFields = winapi::um::wingdi::DM_BITSPERPEL
                | winapi::um::wingdi::DM_PELSWIDTH
                | winapi::um::wingdi::DM_PELSHEIGHT;
            dev_mode.dmBitsPerPel = 32;
            dev_mode.dmPelsWidth = 1920;
            dev_mode.dmPelsHeight = 1080;
            if winapi::um::winuser::ChangeDisplaySettingsA(
                &mut dev_mode,
                winapi::um::winuser::CDS_FULLSCREEN,
            ) != winapi::um::winuser::DISP_CHANGE_SUCCESSFUL
            {
                return (0 as HWND, 0 as HDC);
            }
            winapi::um::winuser::ShowCursor(0);

            h_wnd = CreateWindowExA(
                0,
                "static\0".as_ptr() as *const i8, // class we registered.
                "GLWIN\0".as_ptr() as *const i8,  // title
                WS_POPUP | WS_VISIBLE | WS_MAXIMIZE,
                0,
                0,
                0,
                0,              // size and position
                0 as HWND,      // hWndParent
                0 as HMENU,     // hMenu
                0 as HINSTANCE, // hInstance
                0 as LPVOID,
            ); // lpParam
        }

        #[cfg(not(feature = "fullscreen"))]
        {
            let hinstance = GetModuleHandleA(0 as *const i8);
            let mut wnd_class: WNDCLASSA = core::mem::zeroed();
            wnd_class.style = CS_OWNDC | CS_HREDRAW | CS_VREDRAW;
            wnd_class.lpfnWndProc = Some(window_proc);
            wnd_class.hInstance = hinstance; // The instance handle for our application which we can retrieve by calling GetModuleHandleW.
            wnd_class.lpszClassName = "MyClass\0".as_ptr() as *const i8;
            RegisterClassA(&wnd_class);

            h_wnd = CreateWindowExA(
                0,
                //WS_EX_APPWINDOW | WS_EX_WINDOWEDGE,                     // dwExStyle
                "MyClass\0".as_ptr() as *const i8, // class we registered.
                "GLWIN\0".as_ptr() as *const i8,   // title
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,  // dwStyle
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                1920,
                1080,       // size and position
                0 as HWND,  // hWndParent
                0 as HMENU, // hMenu
                hinstance,  // hInstance
                0 as LPVOID,
            ); // lpParam
        }
        let h_dc: HDC = GetDC(h_wnd); // Device Context

        let mut pfd: PIXELFORMATDESCRIPTOR = core::mem::zeroed();
        pfd.nSize = core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16;
        pfd.nVersion = 1;
        pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
        pfd.iPixelType = PFD_TYPE_RGBA;
        pfd.cColorBits = 32;
        pfd.cAlphaBits = 8;
        pfd.cDepthBits = 32;

        #[cfg(feature = "logger")]
        {
            let pf_id: i32 = ChoosePixelFormat(h_dc, &pfd);
            if pf_id == 0 {
                show_error("ChoosePixelFormat() failed.\0".as_ptr() as *const i8);
                return (0 as HWND, h_dc);
            }

            if SetPixelFormat(h_dc, pf_id, &pfd) == 0 {
                show_error("SetPixelFormat() failed.\0".as_ptr() as *const i8);
                return (0 as HWND, h_dc);
            }

            let gl_context: HGLRC = wglCreateContext(h_dc); // Rendering Contex
            if gl_context == 0 as HGLRC {
                show_error("wglCreateContext() failed.\0".as_ptr() as *const i8);
                return (0 as HWND, h_dc);
            }

            if wglMakeCurrent(h_dc, gl_context) == 0 {
                show_error("wglMakeCurrent() failed.\0".as_ptr() as *const i8);
                return (0 as HWND, h_dc);
            }
        }

        /*#[cfg(not(feature = "logger"))]
        {
            let pf_id: i32 = ChoosePixelFormat(h_dc, &pfd);
            SetPixelFormat(h_dc, pf_id, &pfd);
            let gl_context: HGLRC = wglCreateContext(h_dc); // Rendering Context
            wglMakeCurrent(h_dc, gl_context);
        }

        // make the system font the device context's selected font
        winapi::um::wingdi::SelectObject(
            h_dc,
            winapi::um::wingdi::GetStockObject(winapi::um::wingdi::SYSTEM_FONT as i32),
        );

        // create the bitmap display lists
        winapi::um::wingdi::wglUseFontBitmapsA(h_dc, 0, 255, 1000);*/

        //gl::init();
        //gl::wglSwapIntervalEXT(1);
        (h_wnd, h_dc)
    }
}

// Create message handling function with which to link to hook window to Windows messaging system
// More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms644927(v=vs.85).aspx
pub fn handle_message(_window: HWND) -> bool {
    unsafe {
        let mut msg: MSG = MaybeUninit::uninit().assume_init();
        loop {
            if PeekMessageA(&mut msg, 0 as HWND, 0, 0, PM_REMOVE) == 0 {
                return true;
            }
            if msg.message == winapi::um::winuser::WM_QUIT {
                return false;
            }
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *((dest as usize + i) as *mut u8) = c as u8;
        i += 1;
    }
    dest
}

#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *((dest as usize + i) as *mut u8) = *((src as usize + i) as *const u8);
        i += 1;
    }
    dest
}

#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;

    while i < n {
        let a = *((s1 as usize + i) as *const u8);
        let b = *((s2 as usize + i) as *const u8);
        if a != b {
            return a as i32 - b as i32;
        }
        i += 1;
    }

    0
}

//this is fine
#[cfg(debug_assertions)]
#[no_mangle]
extern "C" fn __chkstk() {}
