//! Aether GUI by NetRepublic — a Windows front end for the Aether core.
//!
//! Made by Amirreza. The tunnelling engine is CluvexStudio's work
//! (https://github.com/CluvexStudio/Aether); this crate is the window around it.

// Hide the console window in release builds. Debug builds keep it, because the core
// logs a great deal of useful detail while a tunnel is coming up.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod autostart;
mod commands;
mod engine;
mod model;
mod proxy_mode;
mod relay;
mod settings_store;
mod stats;
mod tun;

use commands::AppState;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WindowEvent};

fn main() {
    // The core logs through `log`, so its output lands wherever ours does.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,aether=info"))
        .format_timestamp_millis()
        .try_init()
        .ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // A second instance would fight the first over the route table and the proxy
        // registry keys, so hand focus back to the existing window instead.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            reveal(app);
        }))
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::disconnect,
            commands::load_settings,
            commands::save_settings,
            commands::scanned_peers,
            commands::is_elevated,
            commands::core_version,
            commands::open_url,
            commands::win_minimize,
            commands::win_hide,
        ])
        .setup(|app| {
            build_tray(app.handle())?;

            // `--minimized` is what the start-with-Windows entry passes, so the app
            // does not steal focus at sign-in.
            if std::env::args().any(|arg| arg == "--minimized") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing while connected would strand the proxy settings and the routes.
            // Hide instead; quitting goes through the tray, which tears down first.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("could not start Aether GUI");
}

/// Bring the main window back to the foreground.
fn reveal(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Aether GUI", true, None::<&str>)?;
    let disconnect = MenuItem::with_id(app, "disconnect", "Disconnect", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &disconnect, &quit])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("icons/icon.png".into()))?;

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Aether GUI by NetRepublic")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => reveal(app),

            "disconnect" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    commands::shutdown_everything(&app.state::<AppState>()).await;
                });
            }

            "quit" => {
                let app = app.clone();
                // Restore the machine's networking *before* exiting. A hard exit here
                // would leave the proxy or the route table pointing at a dead process.
                tauri::async_runtime::spawn(async move {
                    commands::shutdown_everything(&app.state::<AppState>()).await;
                    app.exit(0);
                });
            }

            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                reveal(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
