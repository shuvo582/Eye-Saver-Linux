use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static INHIBIT: Mutex<Option<Child>> = Mutex::new(None);
static ACTIVE: AtomicBool = AtomicBool::new(false);

/// Prevent display sleep while enabled (Linux: systemd-inhibit).
pub fn set_prevent_sleep(enabled: bool) {
    if enabled && !ACTIVE.swap(true, Ordering::SeqCst) {
        let child = Command::new("systemd-inhibit")
            .args([
                "--what=sleep:idle",
                "--who=EyeSaver",
                "--why=Eye Saver break timer active",
                "--mode=block",
                "sleep",
                "infinity",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Ok(c) = child {
            *INHIBIT.lock().unwrap() = Some(c);
        } else {
            ACTIVE.store(false, Ordering::SeqCst);
        }
    } else if !enabled && ACTIVE.swap(false, Ordering::SeqCst) {
        if let Some(mut c) = INHIBIT.lock().unwrap().take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

pub fn shutdown() {
    set_prevent_sleep(false);
}
