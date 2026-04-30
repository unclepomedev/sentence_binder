use std::time::Duration;

// --- Keyboard Monitor Configuration ---------------------------------------------------------------
/// Maximum time allowed between two 'C' presses to register as a double-tap
pub const DOUBLE_TAP_THRESHOLD: Duration = Duration::from_millis(400);
/// Cooldown period used to reset the timer and prevent triple-taps from firing twice
pub const CLIPBOARD_READ_DELAY: Duration = Duration::from_millis(100);
/// Maximum time allowed for the `osascript` invocation that resolves the active
/// browser/app context. If exceeded, capture proceeds without context.
pub const OSASCRIPT_TIMEOUT: Duration = Duration::from_millis(1500);

// --- Database Configuration ----------------------------------------------------------------------
pub const DB_NAME: &str = "sentence_binder.db";
pub const MAX_DB_CONNECTIONS: u32 = 5;

// --- IPC events ----------------------------------------------------------------------------------
/// Must match `CAPTURE_TRIGGERED` in `src/types/ipc.ts`.
pub const EVENT_CAPTURE_TRIGGERED: &str = "double-tap-cmd-c";
