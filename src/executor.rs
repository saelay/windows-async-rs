use std::future::{Future};
use std::pin::{Pin};
use std::task::{Context, Poll, Waker, RawWaker, RawWakerVTable};

use log;

use windows::core::{
    HSTRING,
    PCWSTR,
};

/* required feature "Win32_Foundation" */
use windows::Win32::Foundation::{
    HWND,
    WPARAM, LPARAM, LRESULT,
    GetLastError,
};

/* required feature "Win32_System_LibraryLoader" */
use windows::Win32::System::LibraryLoader::{
    GetModuleHandleW,
};

/* required feature "Win32_UI_WindowsAndMessaging" */
use windows::Win32::UI::WindowsAndMessaging::{
    WNDCLASSEXW,
    RegisterClassExW, /* required feature "Win32_Graphics_Gdi" */
    CreateWindowExW,
    DestroyWindow,
    SetForegroundWindow,
    WS_POPUP,
    WS_EX_LEFT,
    HMENU,
    MSG,
    GetMessageW,
    TranslateMessage,
    DispatchMessageW,
    WM_USER,
    CS_HREDRAW,
    CS_VREDRAW,
    DefWindowProcW,
    IDI_APPLICATION,
    IDC_ARROW,
    HICON,
    COLOR_WINDOW,
    LoadIconW,
    LoadCursorW,
    PostMessageW,
};

use windows::Win32::Graphics::Gdi::{
    HBRUSH,
};

const VTABLE: RawWakerVTable = RawWakerVTable::new(
    /* clone */
    |dummy_window| {
        RawWaker::new(dummy_window, &VTABLE)
    },

    /* wake */
    |dummy_window| {
        let dummy_window:&DummyWindow = unsafe { &*(dummy_window as *const DummyWindow) };
        crate::executor::post_message_asyncop_completed(dummy_window);
    },

    /* wake_by_ref */
    |dummy_window| {
        let dummy_window:&DummyWindow = unsafe { &*(dummy_window as *const DummyWindow) };
        crate::executor::post_message_asyncop_completed(dummy_window);
    },

    /* drop */
    |_| {},
);

const DUMMY_WINDOW_CLASSNAME:&str = "windows-msgloop-async:DummyWindow";
const DUMMY_WINDOW_WINDOWNAME:&str = "windows-msgloop-async:DummyWindow";
const WM_USER_ASYNCOP_COMPLETED:u32 = WM_USER + 1;

static mut REGISTER_ATOM: Option<u16> = None;


fn post_message_asyncop_completed(dummy_window: &DummyWindow) {
    unsafe {
        log::trace!("PostMessage(WM_USER_ASYNCOP_COMPLETED) to {:?}", dummy_window);
        PostMessageW(dummy_window.hwnd(), WM_USER_ASYNCOP_COMPLETED, WPARAM(0), LPARAM(0));
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_USER_ASYNCOP_COMPLETED => {
            log::trace!("AsyncOperation/AsyncAction Completed ({:?})", hwnd);
        }
        _ => {}
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Run a future to completion on the current thread.
/// 
/// This function will run message-loop, until the given future has completed.
pub fn block_on<T>(mut f: impl Future<Output=T>) -> T {
    
    // Create dummy window to receive window message.
    let wnd = create_dummy_window();
    let pwnd: *const DummyWindow = &wnd;

    let waker = unsafe { Waker::from_raw(RawWaker::new(pwnd as *const (), &VTABLE)) };
    let mut ctx = Context::from_waker(&waker);

    let mut msg = MSG::default();
    loop {
        let f = unsafe { Pin::new_unchecked(&mut f) };
        if let Poll::Ready(ret) = Future::poll(f, &mut ctx) {
            return ret;
        }

        unsafe {
            GetMessageW(&mut msg, None, 0, 0);
            TranslateMessage(&mut msg);
            DispatchMessageW(&mut msg);
        }
    }
}

/// Wrapper type of invisible window handle.
#[derive(Debug)]
#[repr(transparent)]
pub struct DummyWindow(HWND);

impl DummyWindow {
    /// Get `HWND` of window.
    pub fn hwnd(&self) -> HWND {
        self.0.clone()
    }
}

impl Drop for DummyWindow {
    /// Destroy window by calling `DestroyWindow`.
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.0);
        }
    }
}

fn register_dummy_window() {
    unsafe {
        let hmodule = GetModuleHandleW(PCWSTR(std::ptr::null()));
    
        let wndcls = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hmodule,
            hIcon: LoadIconW(hmodule.clone(), IDI_APPLICATION),
            hCursor: LoadCursorW(hmodule.clone(), IDC_ARROW),
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as isize),
            lpszMenuName: PCWSTR(std::ptr::null()),
            lpszClassName: PCWSTR(&HSTRING::from(DUMMY_WINDOW_CLASSNAME).as_wide()[0]),
            hIconSm: HICON::default(),
        };
    
        let ret = RegisterClassExW(&wndcls);
        if ret == 0 {
            let err = GetLastError();
            log::debug!("RegisterClassExW failed (last_error={:?})", err);
        }
        else {
            REGISTER_ATOM = Some(ret);
        }
    }
}

/// Create invisible dummy window.
pub fn create_dummy_window() -> DummyWindow {
    unsafe {
        if REGISTER_ATOM.is_none() {
            register_dummy_window();
        }
        let hmodule = GetModuleHandleW(PCWSTR(std::ptr::null()));
        let hwnd = CreateWindowExW(
            WS_EX_LEFT,
            PCWSTR(&HSTRING::from(DUMMY_WINDOW_CLASSNAME).as_wide()[0]),
            PCWSTR(&HSTRING::from(DUMMY_WINDOW_WINDOWNAME).as_wide()[0]),
            WS_POPUP,
            1, 1, 1, 1,
            HWND::default(),
            HMENU::default(),
            hmodule,
            std::ptr::null_mut(),
        );

        if hwnd.0 == 0 {
            let err = GetLastError();
            log::debug!("CreateWindowExW failed (last_error={:?})", err);
        }
        else {
            log::trace!("Create Dummy window ({:?})", hwnd);
        }

        SetForegroundWindow(hwnd);
        DummyWindow(hwnd)
    }
}
