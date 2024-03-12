// use std::sync::Mutex;

use anyhow::{bail, Result};
use tauri::{App, AppHandle};
use windows::Win32::{
    Foundation::HWND,
    UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
    UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW,
        TranslateMessage,
        EVENT_OBJECT_DESTROY, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MOVESIZEEND, MSG, OBJID_WINDOW,
        WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
    },
};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowInfo, WINDOWINFO, WS_VISIBLE};

#[derive(Debug)]
pub enum UpdateEvents {
    Active(HWND),
    Move(HWND),
    // Create(HWND),
    Destroy(HWND),
}

pub struct ApplicationPlugin {
}

impl crate::plugins::application::ApplicationPlugin {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&mut self) -> Result<()> {
        let _handle = std::thread::spawn(|| unsafe {
            let e = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_OBJECT_DESTROY,
                None,
                Some(event_hook_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            );

            if e.is_invalid() {
                bail!("SetWinEventHook failed");
            }

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let true = UnhookWinEvent(e).as_bool() else {
                bail!("UnhookWinEvent failed");
            };

            Ok(())
        });

        Ok(())
    }
}

unsafe extern "system" fn event_hook_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _dw_event_thread: u32,
    _dwms_event_time: u32,
) {
    let ue = match event {
        EVENT_SYSTEM_FOREGROUND => UpdateEvents::Active(hwnd),
        EVENT_SYSTEM_MOVESIZEEND => UpdateEvents::Move(hwnd),
        // EVENT_OBJECT_DESTROY if id_object == OBJID_WINDOW.0 => UpdateEvents::Destroy(hwnd),
        _ => return,
    };

    println!("event_hook_proc: {:?}", ue);

    let _ = check_active_window(hwnd);
}

unsafe fn check_active_window(hwnd: HWND) -> Result<()> {
    let mut buf = [0u16; 1024];
    let len = unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowTextW(hwnd, &mut buf) };
    let title = String::from_utf16_lossy(&buf[..len as usize]);

    let mut info = WINDOWINFO {
        cbSize: core::mem::size_of::<WINDOWINFO>() as u32,
        ..Default::default()
    };
    unsafe {
        GetWindowInfo(hwnd, &mut info).unwrap() // destroyではすでにアプリは消えていて、これはpanicになることがある。
    };

    if !title.is_empty() && info.dwStyle.contains(WS_VISIBLE) {
        println!(
            "{} ({}, {})-({}, {})",
            title,
            info.rcWindow.left, info.rcWindow.top,
            info.rcWindow.right, info.rcWindow.bottom,
        );
    }

    Ok(())
}
