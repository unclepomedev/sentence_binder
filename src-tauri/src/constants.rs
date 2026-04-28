use std::time::Duration;

// --- Keyboard Monitor Configuration ---------------------------------------------------------------
/// Maximum time allowed between two 'C' presses to register as a double-tap
pub const DOUBLE_TAP_THRESHOLD: Duration = Duration::from_millis(400);
/// Cooldown period used to reset the timer and prevent triple-taps from firing twice
pub const CLIPBOARD_READ_DELAY: Duration = Duration::from_millis(100);

// --- Database Configuration ----------------------------------------------------------------------
pub const DB_NAME: &str = "sentence_binder.db";
pub const MAX_DB_CONNECTIONS: u32 = 5;

// --- IPC events ----------------------------------------------------------------------------------
/// Must match `CAPTURE_TRIGGERED` in `src/types/ipc.ts`.
pub const EVENT_CAPTURE_TRIGGERED: &str = "double-tap-cmd-c";

// --- Messages ------------------------------------------------------------------------------------
/// Sentinel error message returned when no credential entry exists for the requested provider.
pub const KEY_NOT_FOUND_MESSAGE: &str = "Key not found";
