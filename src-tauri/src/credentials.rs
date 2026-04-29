use keyring_core::{Entry, Error};

const SERVICE_NAME: &str = "sentence_binder_secure_vault";

#[derive(Debug)]
pub struct CredentialError(pub String);

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
pub fn save_key(provider: LlmProvider, key: &str) -> Result<(), CredentialError> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(CredentialError("API key cannot be empty".to_string()));
    }

    let entry = Entry::new(SERVICE_NAME, provider.as_str())
        .map_err(|e| CredentialError(format!("Keychain access failed: {}", e)))?;

    entry
        .set_password(trimmed)
        .map_err(|e| CredentialError(format!("Failed to save key: {}", e)))
}

/// Checks whether an API key exists for the given provider.
///
/// This safely confirms existence without returning the secret value to the caller.
pub fn has_key(provider: LlmProvider) -> Result<bool, CredentialError> {
    let entry = Entry::new(SERVICE_NAME, provider.as_str())
        .map_err(|e| CredentialError(format!("Keychain access failed: {}", e)))?;

    entry_exists(&entry)
}

/// Returns `true` if the keychain entry has a password, `false` if it has none.
///
/// `keyring_core` does not expose a native existence-check API, so this calls
/// `get_password` and immediately drops the returned string. The secret is
/// never logged, returned, or otherwise observed.
fn entry_exists(entry: &Entry) -> Result<bool, CredentialError> {
    match entry.get_password() {
        Ok(secret) => {
            drop(secret);
            Ok(true)
        }
        Err(Error::NoEntry) => Ok(false),
        Err(e) => Err(CredentialError(format!("Keychain error: {}", e))),
    }
}

/// Deletes an API key from the macOS Keychain.
///
/// This operation is idempotent; it will return `Ok(())` even if the key
/// was already deleted or never existed.
pub fn delete_key(provider: LlmProvider) -> Result<(), CredentialError> {
    let entry = Entry::new(SERVICE_NAME, provider.as_str())
        .map_err(|e| CredentialError(format!("Keychain access failed: {}", e)))?;

    match entry.delete_credential() {
        Ok(_) | Err(Error::NoEntry) => Ok(()),
        Err(e) => Err(CredentialError(format!("Failed to delete key: {}", e))),
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
        assert_eq!(LlmProvider::Google.as_str(), "google_api_key");
        assert_eq!(LlmProvider::Local.as_str(), "local_api_key");
    }

    #[test]
    fn test_empty_key_rejection() {
        let result = save_key(LlmProvider::Local, "   ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "API key cannot be empty");
    }

    // Test-only helpers that mirror the public save/has/delete API but accept
    // an arbitrary `service` string. This lets the integration test use a
    // dedicated test namespace so it cannot read, overwrite, or delete any
    // production credential stored under `SERVICE_NAME`.
    fn keychain_entry_for(service: &str, account: &str) -> Result<Entry, CredentialError> {
        Entry::new(service, account)
            .map_err(|e| CredentialError(format!("Keychain access failed: {}", e)))
    }

    fn validate_api_key(key: &str) -> Result<&str, CredentialError> {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return Err(CredentialError("API key cannot be empty".to_string()));
        }
        Ok(trimmed)
    }

    fn save_key_in(service: &str, account: &str, key: &str) -> Result<(), CredentialError> {
        let trimmed = validate_api_key(key)?;
        let entry = keychain_entry_for(service, account)?;
        entry
            .set_password(trimmed)
            .map_err(|e| CredentialError(format!("Failed to save key: {}", e)))
    }

    fn has_key_in(service: &str, account: &str) -> Result<bool, CredentialError> {
        let entry = keychain_entry_for(service, account)?;
        entry_exists(&entry)
    }

    fn delete_key_in(service: &str, account: &str) -> Result<(), CredentialError> {
        let entry = keychain_entry_for(service, account)?;
        match entry.delete_credential() {
            Ok(_) | Err(Error::NoEntry) => Ok(()),
            Err(e) => Err(CredentialError(format!("Failed to delete key: {}", e))),
        }
    }

    /// Integration test: performs a real keychain round-trip.
    ///
    /// Requires a configured default `keyring_core` store (e.g. the macOS
    /// Apple Keychain store initialized in `lib.rs`). Ignored by default so
    /// it doesn't run in normal CI; run with:
    ///   cargo test -- --ignored test_keychain_round_trip
    ///
    /// Uses a test-only service/account namespace so the production
    /// `SERVICE_NAME` + `LlmProvider::*` slots are never touched.
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

        // Dedicated test namespace — must not collide with the production
        // `SERVICE_NAME` constant or any real provider account name.
        const TEST_SERVICE: &str = "sentence_binder_secure_vault__test";
        let test_account = format!("test_account_{}", uuid::Uuid::new_v4());
        let secret = format!("test-secret-{}", uuid::Uuid::new_v4());

        // Catch panics to ensure the teardown block always runs.
        let outcome = std::panic::catch_unwind(|| {
            assert!(
                save_key_in(TEST_SERVICE, &test_account, &secret).is_ok(),
                "save_key failed"
            );
            assert!(
                has_key_in(TEST_SERVICE, &test_account).expect("has_key failed"),
                "key should exist after save"
            );
            assert!(
                delete_key_in(TEST_SERVICE, &test_account).is_ok(),
                "delete_key failed"
            );
            assert!(
                !has_key_in(TEST_SERVICE, &test_account).expect("has_key failed"),
                "key should be absent after delete"
            );
        });

        // Teardown: ensure the synthetic secret never leaks into the Keychain.
        let _ = delete_key_in(TEST_SERVICE, &test_account);

        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
}
