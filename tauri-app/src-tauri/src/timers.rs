use crate::settings::{self, Settings, now_ms};
use crate::windows;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reason {
    None,
    Break,
    Micro,
}

pub struct TimerState {
    pub remaining: AtomicU64,
    pub active_reason: std::sync::Mutex<Reason>,
    pub running: AtomicBool,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            remaining: AtomicU64::new(0),
            active_reason: std::sync::Mutex::new(Reason::None),
            running: AtomicBool::new(false),
        }
    }
}

pub struct Shared {
    pub settings: std::sync::Mutex<Settings>,
    pub timer: TimerState,
}

impl Shared {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            settings: std::sync::Mutex::new(settings::load()),
            timer: TimerState::default(),
        })
    }
}

pub fn open_break(app: &AppHandle, reason: &str) {
    let shared = app.state::<Arc<Shared>>();
    let s = shared.settings.lock().unwrap().clone();

    if windows::break_open(app) {
        return;
    }

    let remaining: u64 = if reason == "micro" {
        s.micro_pause_duration_seconds.max(1) as u64
    } else {
        (s.break_duration_minutes.max(1) * 60) as u64
    };

    shared.timer.remaining.store(remaining, Ordering::SeqCst);
    *shared.timer.active_reason.lock().unwrap() = if reason == "micro" {
        Reason::Micro
    } else {
        Reason::Break
    };
    shared.timer.running.store(true, Ordering::SeqCst);

    let message = if reason == "micro" {
        "Micro pause — blink and relax your eyes".to_string()
    } else {
        settings::pick_reminder(&s)
    };

    let _ = windows::show_break(app, reason, remaining, &message);

    if s.micro_pauses_sound && reason == "micro" {
        use std::io::Write;
        let mut stdout = std::io::stdout();
        let _ = stdout.write_all(b"\x07");
        let _ = stdout.flush();
    }
}

pub fn close_break(app: &AppHandle) {
    let shared = app.state::<Arc<Shared>>();
    let reason = *shared.timer.active_reason.lock().unwrap();
    shared.timer.running.store(false, Ordering::SeqCst);
    *shared.timer.active_reason.lock().unwrap() = Reason::None;
    windows::close_break(app);

    if let Ok(mut s) = shared.settings.lock() {
        match reason {
            Reason::Break => s.last_break_at = now_ms(),
            Reason::Micro => s.last_micro_at = now_ms(),
            Reason::None => {}
        }
        let _ = settings::save(&s);
    }
}

pub fn delay_break(app: &AppHandle, minutes: u32) {
    let shared = app.state::<Arc<Shared>>();
    {
        let mut s = shared.settings.lock().unwrap();
        let interval = s.break_interval_minutes.max(1);
        let mins = minutes.min(interval);
        s.last_break_at = now_ms() - (interval - mins) as u64 * 60_000;
        let _ = settings::save(&s);
    }
    close_break(app);
}

pub fn skip_break(app: &AppHandle) {
    let shared = app.state::<Arc<Shared>>();
    {
        let s = shared.settings.lock().unwrap();
        if !s.enable_skipping {
            return;
        }
    }
    {
        let mut s = shared.settings.lock().unwrap();
        s.last_break_at = now_ms();
        let _ = settings::save(&s);
    }
    close_break(app);
}

pub fn apply_all(app: &AppHandle) {
    let shared = app.state::<Arc<Shared>>();
    let s = shared.settings.lock().unwrap().clone();
    let _ = settings::save(&s);
    windows::apply_effects(app, &s);
    crate::power::set_prevent_sleep(s.prevent_sleeping);
    crate::autostart::set_autostart(s.autostart);
    if let Some(idle) = app.try_state::<crate::idle::IdleFlags>() {
        idle.set_reset(s.reset_timers_after_idle);
        idle.set_threshold(s.idle_reset_minutes);
    }
    schedule_now(app);
}

pub fn schedule_now(app: &AppHandle) {
    let shared = app.state::<Arc<Shared>>();
    let s = shared.settings.lock().unwrap().clone();
    let now = now_ms();

    if windows::break_open(app) {
        return;
    }

    if s.breaks_enabled {
        let elapsed = now.saturating_sub(s.last_break_at);
        let interval = s.break_interval_minutes.max(1) as u64 * 60_000;
        if elapsed >= interval {
            open_break(app, "break");
            return;
        }
    }

    if s.micro_pauses_enabled {
        let elapsed = now.saturating_sub(s.last_micro_at);
        let interval = s.micro_pause_interval_minutes.max(1) as u64 * 60_000;
        if elapsed >= interval {
            open_break(app, "micro");
        }
    }
}

pub fn spawn_ticker(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        let shared = app.state::<Arc<Shared>>();
        if shared.timer.running.load(Ordering::SeqCst) && windows::break_open(&app) {
            let rem = shared
                .timer
                .remaining
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                    Some(v.saturating_sub(1))
                })
                .unwrap_or(0);
            windows::emit_tick(&app, rem);
            if rem == 0 {
                close_break(&app);
            }
        } else if !windows::break_open(&app) {
            schedule_now(&app);
        }
    });
}

pub fn reset_timers_idle(app: &AppHandle) {
    let shared = app.state::<Arc<Shared>>();
    if let Ok(mut s) = shared.settings.lock() {
        s.last_break_at = now_ms();
        s.last_micro_at = now_ms();
        let _ = settings::save(&s);
    }
}
