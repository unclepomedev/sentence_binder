use crate::constants::KEY_NOT_FOUND_MESSAGE;
use keyring_core::{Entry, Error};

const SERVICE_NAME: &str = "sentence_binder_secure_vault";

#[derive(Debug, PartialEq)]
pub enum LlmProvider {
    OpenAi,
    Anthropic,
    Google,
    Local,
}

impl LlmProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAi => "openai_api_key",
            Self::Anthropic => "anthropic_api_key",
            Self::Google => "google_api_key",
            Self::Local => "local_api_key",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "openai" => Some(Self::OpenAi),
            "anthropic" => Some(Self::Anthropic),
            "google" => Some(Self::Google),
            "local" => Some(Self::Local),
            _ => None,
        }
    }
}

/// Saves the API key to the macOS Keychain.
///
/// Fails if the provided key string is empty, or if the OS denies access to the Keychain.
pub fn save_key(provider: LlmProvider, key: &str) -> Result<(), String> {
    if key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    let entry = Entry::new(SERVICE_NAME, provider.as_str())
        .map_err(|e| format!("Keychain access failed: {}", e))?;

    entry
        .set_password(key)
        .map_err(|e| format!("Failed to save key: {}", e))
}

/// Retrieves an API key from the macOS Keychain.
///
/// Returns the requested API key, or an error if the key does not exist
/// or the Keychain is inaccessible.
pub fn get_key(provider: LlmProvider) -> Result<String, String> {
    let entry = Entry::new(SERVICE_NAME, provider.as_str())
        .map_err(|e| format!("Keychain access failed: {}", e))?;

    match entry.get_password() {
        Ok(key) => Ok(key),
        Err(Error::NoEntry) => Err(KEY_NOT_FOUND_MESSAGE.to_string()),
        Err(e) => Err(format!("Keychain error: {}", e)),
    }
}

/// Deletes an API key from the macOS Keychain.
///
/// This operation is idempotent; it will return `Ok(())` even if the key
/// was already deleted or never existed.
pub fn delete_key(provider: LlmProvider) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, provider.as_str())
        .map_err(|e| format!("Keychain access failed: {}", e))?;

    match entry.delete_credential() {
        Ok(_) | Err(Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Failed to delete key: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_string_mapping() {
        assert_eq!(LlmProvider::from_str("openai"), Some(LlmProvider::OpenAi));
        assert_eq!(
            LlmProvider::from_str("anthropic"),
            Some(LlmProvider::Anthropic)
        );
        assert_eq!(LlmProvider::from_str("google"), Some(LlmProvider::Google));
        assert_eq!(LlmProvider::from_str("local"), Some(LlmProvider::Local));
        assert_eq!(LlmProvider::from_str("invalid"), None);

        assert_eq!(LlmProvider::OpenAi.as_str(), "openai_api_key");
        assert_eq!(LlmProvider::Anthropic.as_str(), "anthropic_api_key");
    }

    #[test]
    fn test_empty_key_rejection() {
        let result = save_key(LlmProvider::Local, "   ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "API key cannot be empty");
    }

    /// Integration test: performs a real keychain round-trip.
    ///
    /// Requires a configured default `keyring_core` store (e.g. the macOS
    /// Apple Keychain store initialized in `lib.rs`). Ignored by default so
    /// it doesn't run in normal CI; run with:
    ///   cargo test -- --ignored test_keychain_round_trip
    #[test]
    #[ignore]
    fn test_keychain_round_trip() {
        use apple_native_keyring_store::keychain::Store as AppleKeychainStore;
        use std::sync::Once;
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let store = AppleKeychainStore::new().expect("init store");
            keyring_core::set_default_store(store);
        });
        let secret = format!("test-secret-{}", uuid::Uuid::new_v4());

        // Catch panics to ensure the teardown block always runs
        let outcome = std::panic::catch_unwind(|| {
            assert!(
                save_key(LlmProvider::Local, &secret).is_ok(),
                "save_key failed"
            );

            let fetched = get_key(LlmProvider::Local).expect("get_key failed");
            assert_eq!(fetched, secret, "fetched value does not match stored value");

            assert!(delete_key(LlmProvider::Local).is_ok(), "delete_key failed");

            let after = get_key(LlmProvider::Local);
            assert!(after.is_err(), "expected Err after delete");
        });

        // Teardown: always attempt deletion to avoid leaking credentials into the developer's Mac
        let _ = delete_key(LlmProvider::Local);

        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
}
