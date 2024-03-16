use anyhow::{bail, Result};
use once_cell::sync::OnceCell;
use sysinfo;
use tokio::sync::mpsc;
use windows::Win32::{
    Foundation::HWND,
    UI::{Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK}, WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, GetWindowTextW, GetWindowThreadProcessId, TranslateMessage,
        EVENT_OBJECT_DESTROY, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MOVESIZEEND, MSG,
        WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
    }},
};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowInfo, WINDOWINFO, WS_VISIBLE};

use crate::app::db;

#[derive(Debug)]
pub enum UpdateEvents {
    Active(HWND),
    Move(HWND),
    // Destroy(HWND),
}

static UPDATE_EVENTS_TX: OnceCell<mpsc::Sender<UpdateEvents>> = OnceCell::new();

pub struct ApplicationLog {
    name: String,
    title: String,
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
}

pub struct ApplicationPlugin {
}

impl ApplicationPlugin {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&mut self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<UpdateEvents>(100);

        UPDATE_EVENTS_TX.set(tx).unwrap();

        let _manager = tokio::spawn(async move {
            while let Some(ue) = rx.recv().await {
                println!("manager: {:?}", ue);

                match ue {
                    UpdateEvents::Active(hwnd) => {
                        let log = unsafe { check_active_window(hwnd, ue) }.unwrap();
                        sqlx::query!(
                            "INSERT INTO application (eventId, kind, name, title, x0, y0, x1, y1) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                            1,
                            "active",
                            log.name,
                            log.title,
                            log.left,
                            log.top,
                            log.right,
                            log.bottom
                        ).execute(db::pool()).await.unwrap();
                    },
                    UpdateEvents::Move(hwnd) => {
                        let log = unsafe { check_active_window(hwnd, ue) }.unwrap();
                        sqlx::query!(
                            "INSERT INTO application (eventId, kind, name, title, x0, y0, x1, y1) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                            1,
                            "move",
                            log.name,
                            log.title,
                            log.left,
                            log.top,
                            log.right,
                            log.bottom
                        ).execute(db::pool()).await.unwrap();
                    },
                }
            }
        });

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


    if let Some(tx) = UPDATE_EVENTS_TX.get() {
        tx.blocking_send(ue).unwrap_or_else(|e| {
            println!("event_hook_proc: Error on send: {:?}", e);
        });
    }

    // let _ = check_active_window(hwnd, ue);
}

unsafe fn check_active_window(hwnd: HWND, ue: UpdateEvents) -> Result<ApplicationLog> {
    let mut buf: [u16; 1024] = [0u16; 1024];
    let len = GetWindowTextW(hwnd, &mut buf);
    let title = String::from_utf16_lossy(&buf[..len as usize]);

    let mut info = WINDOWINFO {
        cbSize: core::mem::size_of::<WINDOWINFO>() as u32,
        ..Default::default()
    };
    GetWindowInfo(hwnd, &mut info).unwrap(); // destroyではすでにアプリは消えていて、これはpanicになることがある。

    let mut process_id = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));
    if process_id == 0 {
        bail!("GetWindowThreadProcessId failed");
    }

    // let mut sys = sysinfo::System::new_all();
    let mut sys = sysinfo::System::new();
    let pid = sysinfo::Pid::from_u32(process_id);
    sys.refresh_process_specifics(pid, sysinfo::ProcessRefreshKind::new());
    let process = sys.processes().get(&pid);
    let process_name = process.map(|p| p.name().to_string()).unwrap_or_else(|| {
        format!("pid:{}", process_id)
    });

    // let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION, false, process_id).unwrap();
    // if process_handle.is_invalid() {
    //     bail!("OpenProcess failed")
    // }

    // let mut buffer: [u16; 1024] = [0; 1024];
    // let mut size_needed = 0;

    // let result = QueryFullProcessImageNameW(
    //     process_handle,
    //     0,
    //     &mut buffer,
    //     &mut size_needed,
    // );
    // if result.is_false() {
    //     bail!("QueryFullProcessImageNameW failed")
    // }
    // let process_name = String::from_utf16_lossy(&buffer[..size_needed as usize]);
    // CloseHandle(process_handle).unwrap();

    if !title.is_empty() && info.dwStyle.contains(WS_VISIBLE) {
        println!(
            "{}, {} ({}, {})-({}, {})",
            process_name,
            title,
            info.rcWindow.left, info.rcWindow.top,
            info.rcWindow.right, info.rcWindow.bottom,
        );

        // db::execute!(
        //     "INSERT INTO application (eventId, kind, name, title, left, top, right, bottom) VALUES (?, ?, ?, ?, ?, ?)",
        //     1,
        //     "active",
        //     "name",
        //     title,
        //     info.rcWindow.left,
        //     info.rcWindow.top,
        //     info.rcWindow.right,
        //     info.rcWindow.bottom,
        // ).await?;
    }

    Ok(ApplicationLog {
        name: process_name,
        title: title,
        top: info.rcWindow.top,
        right: info.rcWindow.right,
        bottom: info.rcWindow.bottom,
        left: info.rcWindow.left,
    })
}
