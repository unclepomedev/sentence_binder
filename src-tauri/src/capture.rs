use crate::constants;
use arboard::Clipboard;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop, CFRunLoopRun};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    CallbackResult,
};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::async_runtime::spawn;
use tauri::{AppHandle, Emitter, Runtime};
use tokio::time::sleep;

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
        if let Some(last) = self.last_tap {
            if now.duration_since(last) < self.threshold {
                self.last_tap = None;
                return true;
            }
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
                sleep(constants::CLIPBOARD_READ_DELAY).await;
                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        let _ = proxy_clone.emit(constants::EVENT_CAPTURE_TRIGGERED, text);
                    }
                }
            });
        }
    }
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
                eprintln!("[capture] Failed to create runloop source. Double-copy capture will be disabled.");
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
