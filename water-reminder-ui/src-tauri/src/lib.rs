use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use notify_rust::Notification;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};

struct AppState {
    is_running: bool,
    interval_minutes: u64,
}

#[tauri::command]
async fn toggle_reminder(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    enabled: bool,
    minutes: u64,
) -> Result<(), String> {
    let mut state_lock = state.lock().await;
    state_lock.is_running = enabled;
    state_lock.interval_minutes = minutes;

    if enabled {
        let _ = Notification::new()
            .summary("喝水助手已启动")
            .body(&format!("小竹将每隔 {} 分钟提醒你一次。", minutes))
            .show();

        let state_clone = state.inner().clone();
        tokio::spawn(async move {
            loop {
                let mins = {
                    let s = state_clone.lock().await;
                    if !s.is_running { break; }
                    s.interval_minutes
                };
                sleep(Duration::from_secs(mins * 60)).await;
                if !state_clone.lock().await.is_running { break; }
                let _ = Notification::new()
                    .summary("该喝水啦宝宝")
                    .body("墨竹提醒您：喝水不积极，身体出问题")
                    .show();
            }
        });
    } else {
        let _ = Notification::new()
            .summary("喝水助手已关闭")
            .show();
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(Mutex::new(AppState {
        is_running: false,
        interval_minutes: 40,
    }));

    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "退出程序", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "显示界面", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false) // 关键：左键不弹出菜单，留给右键
                .on_menu_event(move |app_handle, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            // 彻底退出程序，不再拦截关闭事件
                            app_handle.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // 只在“左键双击”或“左键单击”时显示界面
                    // 这样右键点击时就只会弹出菜单，而不会弹窗
                    match event {
                        TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } => {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![toggle_reminder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}