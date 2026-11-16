use crate::settings::{self, Settings};
use crate::timers::{self, Shared};
use serde_json::Value;
use std::sync::Arc;
use tauri::{Manager, State};

fn merge(s: &mut Settings, patch: Value) {
    if let Some(obj) = patch.as_object() {
        for (k, v) in obj {
            match k.as_str() {
                "breaksEnabled" => {
                    if let Some(x) = v.as_bool() {
                        s.breaks_enabled = x;
                    }
                }
                "breakDurationMinutes" => {
                    if let Some(x) = v.as_u64() {
                        s.break_duration_minutes = x as u32;
                    }
                }
                "breakIntervalMinutes" => {
                    if let Some(x) = v.as_u64() {
                        s.break_interval_minutes = x as u32;
                    }
                }
                "enableSkipping" => {
                    if let Some(x) = v.as_bool() {
                        s.enable_skipping = x;
                    }
                }
                "disableSkippingSeconds" => {
                    if let Some(x) = v.as_u64() {
                        s.disable_skipping_seconds = x as u32;
                    }
                }
                "microPausesEnabled" => {
                    if let Some(x) = v.as_bool() {
                        s.micro_pauses_enabled = x;
                    }
                }
                "microPauseIntervalMinutes" => {
                    if let Some(x) = v.as_u64() {
                        s.micro_pause_interval_minutes = x as u32;
                    }
                }
                "microPauseDurationSeconds" => {
                    if let Some(x) = v.as_u64() {
                        s.micro_pause_duration_seconds = x as u32;
                    }
                }
                "microPausesSound" => {
                    if let Some(x) = v.as_bool() {
                        s.micro_pauses_sound = x;
                    }
                }
                "blueFilterEnabled" => {
                    if let Some(x) = v.as_bool() {
                        s.blue_filter_enabled = x;
                    }
                }
                "blueFilterIntensity" => {
                    if let Some(x) = v.as_u64() {
                        s.blue_filter_intensity = x as u32;
                    }
                }
                "grayscaleEnabled" => {
                    if let Some(x) = v.as_bool() {
                        s.grayscale_enabled = x;
                    }
                }
                "invertEnabled" => {
                    if let Some(x) = v.as_bool() {
                        s.invert_enabled = x;
                    }
                }
                "overlayEnabled" => {
                    if let Some(x) = v.as_bool() {
                        s.overlay_enabled = x;
                    }
                }
                "overlayOpacity" => {
                    if let Some(x) = v.as_f64() {
                        s.overlay_opacity = x;
                    }
                }
                "overlayColor" => {
                    if let Some(x) = v.as_str() {
                        s.overlay_color = x.to_string();
                    }
                }
                "autoSetMaxBrightness" => {
                    if let Some(x) = v.as_bool() {
                        s.auto_set_max_brightness = x;
                    }
                }
                "preventSleeping" => {
                    if let Some(x) = v.as_bool() {
                        s.prevent_sleeping = x;
                    }
                }
                "autostart" => {
                    if let Some(x) = v.as_bool() {
                        s.autostart = x;
                    }
                }
                "resetTimersAfterIdle" => {
                    if let Some(x) = v.as_bool() {
                        s.reset_timers_after_idle = x;
                    }
                }
                "idleResetMinutes" => {
                    if let Some(x) = v.as_u64() {
                        s.idle_reset_minutes = x as u32;
                    }
                }
                "reminders" => {
                    if let Some(obj) = v.as_object() {
                        if let Some(x) = obj.get("stretch").and_then(|a| a.as_bool()) {
                            s.reminders.stretch = x;
                        }
                        if let Some(x) = obj.get("water").and_then(|a| a.as_bool()) {
                            s.reminders.water = x;
                        }
                    }
                }
                "lastBreakAt" => {
                    if let Some(x) = v.as_u64() {
                        s.last_break_at = x;
                    }
                }
                "lastMicroAt" => {
                    if let Some(x) = v.as_u64() {
                        s.last_micro_at = x;
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn to_camel(s: &Settings) -> Value {
    serde_json::json!({
        "breaksEnabled": s.breaks_enabled,
        "breakDurationMinutes": s.break_duration_minutes,
        "breakIntervalMinutes": s.break_interval_minutes,
        "enableSkipping": s.enable_skipping,
        "disableSkippingSeconds": s.disable_skipping_seconds,
        "microPausesEnabled": s.micro_pauses_enabled,
        "microPauseIntervalMinutes": s.micro_pause_interval_minutes,
        "microPauseDurationSeconds": s.micro_pause_duration_seconds,
        "microPausesSound": s.micro_pauses_sound,
        "blueFilterEnabled": s.blue_filter_enabled,
        "blueFilterIntensity": s.blue_filter_intensity,
        "grayscaleEnabled": s.grayscale_enabled,
        "invertEnabled": s.invert_enabled,
        "overlayEnabled": s.overlay_enabled,
        "overlayOpacity": s.overlay_opacity,
        "overlayColor": s.overlay_color,
        "autoSetMaxBrightness": s.auto_set_max_brightness,
        "preventSleeping": s.prevent_sleeping,
        "autostart": s.autostart,
        "resetTimersAfterIdle": s.reset_timers_after_idle,
        "idleResetMinutes": s.idle_reset_minutes,
        "reminders": {
            "stretch": s.reminders.stretch,
            "water": s.reminders.water
        },
        "lastBreakAt": s.last_break_at,
        "lastMicroAt": s.last_micro_at
    })
}

#[tauri::command]
pub fn get_settings(shared: State<Arc<Shared>>) -> Value {
    let s = shared.settings.lock().unwrap().clone();
    to_camel(&s)
}

#[tauri::command]
pub fn update_settings(app: tauri::AppHandle, shared: State<Arc<Shared>>, patch: Value) -> Value {
    {
        let mut s = shared.settings.lock().unwrap();
        merge(&mut s, patch);
        let _ = settings::save(&s);
    }
    timers::apply_all(&app);
    let s = shared.settings.lock().unwrap().clone();
    to_camel(&s)
}

#[tauri::command]
pub fn reset_settings(app: tauri::AppHandle, shared: State<Arc<Shared>>) -> Value {
    {
        let mut s = Settings::default();
        s.last_break_at = settings::now_ms();
        s.last_micro_at = settings::now_ms();
        *shared.settings.lock().unwrap() = s.clone();
        let _ = settings::save(&s);
    }
    timers::apply_all(&app);
    let s = shared.settings.lock().unwrap().clone();
    to_camel(&s)
}

#[tauri::command]
pub fn preview_break(app: tauri::AppHandle) {
    timers::open_break(&app, "break");
}

#[tauri::command]
pub fn preview_micro(app: tauri::AppHandle) {
    timers::open_break(&app, "micro");
}

#[tauri::command]
pub fn delay_break(app: tauri::AppHandle, minutes: u32) {
    timers::delay_break(&app, minutes);
}

#[tauri::command]
pub fn skip_break(app: tauri::AppHandle) {
    timers::skip_break(&app);
}

#[tauri::command]
pub fn close_break(app: tauri::AppHandle) {
    timers::close_break(&app);
}

#[tauri::command]
pub fn break_tick_info(shared: State<Arc<Shared>>) -> Value {
    let rem = shared
        .timer
        .remaining
        .load(std::sync::atomic::Ordering::SeqCst);
    let active = shared
        .timer
        .running
        .load(std::sync::atomic::Ordering::SeqCst);
    serde_json::json!({
        "remaining": rem,
        "reason": if active { "active" } else { "idle" }
    })
}

#[tauri::command]
pub fn show_main(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}
