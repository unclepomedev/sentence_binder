use crate::constants;
use arboard::Clipboard;
use core_foundation::runloop::{CFRunLoop, CFRunLoopRun, kCFRunLoopCommonModes};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    CallbackResult,
};
use serde::Serialize;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::async_runtime::spawn;
use tauri::{AppHandle, Emitter, Runtime};
use tokio::process::Command;
use tokio::time::{sleep, timeout};

/// represents kCGEventKeyDown
const MAC_KEYDOWN_TYPE: u32 = 10;
/// represents kCGKeyboardEventKeycode
const EVENT_FIELD_KEYCODE: u32 = 9;
/// hardware-independent virtual keycode for the "C" key
const KEYCODE_C: i64 = 8;
/// 2^20, the bitmask for kCGEventFlagMaskCommand
const MAC_CMD_FLAG: u64 = 1_048_576;

lazy_static::lazy_static! {
    static ref DETECTOR: Mutex<DoubleTapDetector> = Mutex::new(DoubleTapDetector::new(constants::DOUBLE_TAP_THRESHOLD));
}

pub struct DoubleTapDetector {
    last_tap: Option<Instant>,
    threshold: Duration,
}

impl DoubleTapDetector {
    pub fn new(threshold: Duration) -> Self {
        Self {
            last_tap: None,
            threshold,
        }
    }

    pub fn register_tap(&mut self, now: Instant) -> bool {
        if let Some(last) = self.last_tap
            && now.duration_since(last) < self.threshold
        {
            self.last_tap = None;
            return true;
        }
        self.last_tap = Some(now);
        false
    }
}

fn handle_event<R: Runtime>(proxy: AppHandle<R>, event_type: CGEventType, event: &CGEvent) {
    if (event_type as u32) != MAC_KEYDOWN_TYPE {
        return;
    }

    let key_code = event.get_integer_value_field(EVENT_FIELD_KEYCODE);
    let flags = event.get_flags();

    if key_code == KEYCODE_C && (flags.bits() & MAC_CMD_FLAG) != 0 {
        let is_double_tap = {
            let mut detector = DETECTOR.lock().unwrap_or_else(|poisoned| {
                eprintln!("[capture] DETECTOR mutex was poisoned; recovering.");
                poisoned.into_inner()
            });
            detector.register_tap(Instant::now())
        };

        if is_double_tap {
            let proxy_clone = proxy.clone();
            spawn(async move {
                let context_fut = get_active_context();
                let (_, context) =
                    tokio::join!(sleep(constants::CLIPBOARD_READ_DELAY), context_fut);
                if let Ok(mut clipboard) = Clipboard::new()
                    && let Ok(text) = clipboard.get_text()
                {
                    let payload = CapturePayload { text, context };
                    if let Err(e) = proxy_clone.emit(constants::EVENT_CAPTURE_TRIGGERED, payload) {
                        eprintln!("[capture] Failed to emit event to frontend: {}", e);
                    }
                }
            });
        }
    }
}

#[derive(Clone, Serialize)]
pub struct CapturePayload {
    pub text: String,
    pub context: Option<String>,
}

/// Uses JS to get the frontmost application.
/// If it's a browser, it attempts to grab the active tab's URL.
///
/// Runs `osascript` asynchronously via `tokio::process::Command` with a timeout so
/// it never blocks the tokio runtime. On error or timeout, returns `None` as a default context.
async fn get_active_context() -> Option<String> {
    let script = include_str!("scripts/get_active_context.js");

    let mut command = Command::new("osascript");
    command.kill_on_drop(true);
    let fut = command
        .arg("-l")
        .arg("JavaScript")
        .arg("-e")
        .arg(script)
        .output();

    let output = match timeout(constants::OSASCRIPT_TIMEOUT, fut).await {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            eprintln!("[capture] osascript failed: {}", e);
            return None;
        }
        Err(_) => {
            eprintln!("[capture] osascript timed out while reading active context");
            return None;
        }
    };

    if !output.status.success() {
        let code = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "<no exit code>".to_string());
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!(
            "[capture] osascript exited with status {}: {}",
            code,
            stderr.trim()
        );
        return None;
    }

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if result.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.trim().is_empty() {
            eprintln!(
                "[capture] osascript produced empty stdout; stderr: {}",
                stderr.trim()
            );
        }
        return None;
    }
    Some(result)
}

/// Initializes a global macOS keyboard event monitor on a dedicated background thread.
///
/// This establishes a `CGEventTap` and binds it to an isolated `CFRunLoop`. Running this
/// loop on a separate thread is mandatory on macOS; attempting to bind it to the main
/// thread would block Tauri's UI render loop and cause the OS to force-kill the application.
pub fn setup_event_tap<R: Runtime>(app_handle: AppHandle<R>) {
    thread::spawn(move || {
        let tap = match CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            vec![CGEventType::KeyDown],
            move |_proxy, event_type, event| {
                handle_event(app_handle.clone(), event_type, event);
                CallbackResult::Keep
            },
        ) {
            Ok(tap) => tap,
            Err(_) => {
                eprintln!(
                    "[capture] Failed to create event tap. \
                     Please grant Accessibility / Input Monitoring permissions \
                     in System Settings and restart the app. \
                     Double-copy capture will be disabled."
                );
                return;
            }
        };

        let loop_source = match tap.mach_port().create_runloop_source(0) {
            Ok(src) => src,
            Err(_) => {
                eprintln!(
                    "[capture] Failed to create runloop source. Double-copy capture will be disabled."
                );
                return;
            }
        };

        let current_loop = CFRunLoop::get_current();
        current_loop.add_source(&loop_source, unsafe { kCFRunLoopCommonModes });
        tap.enable();

        unsafe { CFRunLoopRun() };
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_tap() {
        let mut detector = DoubleTapDetector::new(Duration::from_millis(400));
        let now = Instant::now();

        assert!(!detector.register_tap(now));
    }

    #[test]
    fn test_double_tap_within_threshold() {
        let mut detector = DoubleTapDetector::new(Duration::from_millis(400));
        let first_tap = Instant::now();
        let second_tap = first_tap + Duration::from_millis(200);

        detector.register_tap(first_tap);
        assert!(detector.register_tap(second_tap));
    }

    #[test]
    fn test_double_tap_outside_threshold() {
        let mut detector = DoubleTapDetector::new(Duration::from_millis(400));
        let first_tap = Instant::now();
        let second_tap = first_tap + Duration::from_millis(500);

        detector.register_tap(first_tap);
        assert!(!detector.register_tap(second_tap));
    }

    #[test]
    fn test_triple_tap_resets_state() {
        let mut detector = DoubleTapDetector::new(Duration::from_millis(400));
        let first = Instant::now();
        let second = first + Duration::from_millis(200);
        let third = second + Duration::from_millis(200);

        detector.register_tap(first);
        assert!(detector.register_tap(second));
        assert!(!detector.register_tap(third));
    }
}
