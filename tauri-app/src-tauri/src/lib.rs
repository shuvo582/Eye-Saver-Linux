mod autostart;
mod commands;
mod idle;
mod power;
mod settings;
mod timers;
mod tray;
mod windows;

use idle::IdleFlags;
use std::sync::Arc;
use tauri::Manager;
use timers::Shared;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared = Shared::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .manage(shared)
        .manage(IdleFlags::new(true, 5))
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::update_settings,
            commands::reset_settings,
            commands::preview_break,
            commands::preview_micro,
            commands::delay_break,
            commands::skip_break,
            commands::close_break,
            commands::break_tick_info,
            commands::show_main,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let _ = tray::setup_tray(&handle);

            if let Some(main) = app.get_webview_window("main") {
                let h = handle.clone();
                main.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(w) = h.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                });
            }

            // Sync idle flags from settings
            {
                let shared = app.state::<Arc<Shared>>();
                let s = shared.settings.lock().unwrap().clone();
                let idle = app.state::<IdleFlags>();
                idle.set_reset(s.reset_timers_after_idle);
                idle.set_threshold(s.idle_reset_minutes);
            }

            timers::apply_all(&handle);
            timers::spawn_ticker(handle.clone());

            // Idle watchdog
            let h2 = handle.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(30));
                let idle = h2.state::<IdleFlags>();
                if !idle.load_reset() {
                    continue;
                }
                if let Some(ms) = idle::idle_ms() {
                    let thr = idle.get_threshold() as u64;
                    if ms > thr * 60_000 {
                        timers::reset_timers_idle(&h2);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Eye Saver");
}
