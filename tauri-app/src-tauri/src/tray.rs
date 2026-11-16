use crate::timers::{self, Shared};
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Eye Saver", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let breaks = MenuItem::with_id(app, "breaks", "Breaks: …", true, None::<&str>)?;
    let micro = MenuItem::with_id(app, "micro", "Micro pauses: …", true, None::<&str>)?;
    let blue = MenuItem::with_id(app, "blue", "Blue filter: …", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let take = MenuItem::with_id(app, "take", "Take a break now", true, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&open, &sep1, &breaks, &micro, &blue, &sep2, &take, &sep3, &quit],
    )?;

    // Initial labels from settings
    {
        let shared = app.state::<Arc<Shared>>();
        let s = shared.settings.lock().unwrap().clone();
        let _ = breaks.set_text(if s.breaks_enabled {
            "Breaks: On"
        } else {
            "Breaks: Off"
        });
        let _ = micro.set_text(if s.micro_pauses_enabled {
            "Micro pauses: On"
        } else {
            "Micro pauses: Off"
        });
        let _ = blue.set_text(if s.blue_filter_enabled {
            "Blue filter: On"
        } else {
            "Blue filter: Off"
        });
    }

    let breaks_c = breaks.clone();
    let micro_c = micro.clone();
    let blue_c = blue.clone();

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .tooltip("Eye Saver")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open" => show_main(app),
            "breaks" => {
                {
                    let shared = app.state::<Arc<Shared>>();
                    if let Ok(mut s) = shared.settings.lock() {
                        s.breaks_enabled = !s.breaks_enabled;
                        let _ = crate::settings::save(&s);
                    }
                }
                timers::apply_all(app);
                let s = app.state::<Arc<Shared>>();
                let st = s.settings.lock().unwrap();
                let _ = breaks_c.set_text(if st.breaks_enabled {
                    "Breaks: On"
                } else {
                    "Breaks: Off"
                });
            }
            "micro" => {
                {
                    let shared = app.state::<Arc<Shared>>();
                    if let Ok(mut s) = shared.settings.lock() {
                        s.micro_pauses_enabled = !s.micro_pauses_enabled;
                        let _ = crate::settings::save(&s);
                    }
                }
                timers::apply_all(app);
                let s = app.state::<Arc<Shared>>();
                let st = s.settings.lock().unwrap();
                let _ = micro_c.set_text(if st.micro_pauses_enabled {
                    "Micro pauses: On"
                } else {
                    "Micro pauses: Off"
                });
            }
            "blue" => {
                {
                    let shared = app.state::<Arc<Shared>>();
                    if let Ok(mut s) = shared.settings.lock() {
                        s.blue_filter_enabled = !s.blue_filter_enabled;
                        let _ = crate::settings::save(&s);
                    }
                }
                timers::apply_all(app);
                let s = app.state::<Arc<Shared>>();
                let st = s.settings.lock().unwrap();
                let _ = blue_c.set_text(if st.blue_filter_enabled {
                    "Blue filter: On"
                } else {
                    "Blue filter: Off"
                });
            }
            "take" => timers::open_break(app, "break"),
            "quit" => {
                crate::power::shutdown();
                app.exit(0);
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
                show_main(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

// silence unused type import if Wry not needed
#[allow(dead_code)]
type _Wry = Wry;
