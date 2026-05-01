import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { IpcCommands } from "@/types/ipc";

// Maximum time to wait for the keychain status check before treating the
// IPC call as hung. Without this, a Tauri/IPC deadlock or stalled keychain
// prompt would leave the UI stuck on "Checking..." with no Retry button.
const HAS_API_KEY_TIMEOUT_MS = 8000;

function withTimeout<T>(promise: Promise<T>, ms: number, label: string): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`${label} timed out after ${ms}ms`));
    }, ms);
    promise.then(
      (value) => {
        clearTimeout(timer);
        resolve(value);
      },
      (err) => {
        clearTimeout(timer);
        reject(err);
      },
    );
  });
}

// TODO: temporarily passing openai as provider. change it at a good time
export function useCredentials(provider: string = "openai") {
  const [hasKey, setHasKey] = useState<boolean | null>(null);
  const [isChecking, setIsChecking] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);

  const checkKeyStatus = useCallback(async () => {
    setIsChecking(true);
    try {
      const exists = await withTimeout(
        invoke<boolean>(IpcCommands.HAS_API_KEY, { provider }),
        HAS_API_KEY_TIMEOUT_MS,
        "Key status check",
      );
      setHasKey(exists);
      setError(null);
    } catch (err) {
      console.error("[useCredentials] Failed to check key status:", err);
      if (err instanceof Error) {
        setError(err);
      } else {
        setError(new Error(String(err)));
      }
    } finally {
      setIsChecking(false);
    }
  }, [provider]);

  useEffect(() => {
    void checkKeyStatus();
  }, [checkKeyStatus]);

  const saveKey = async (key: string) => {
    await invoke(IpcCommands.SAVE_API_KEY, { provider, key });
    setHasKey(true);
    setError(null);
    // Re-validate against the real keychain state in case the optimistic
    // update drifts from what the OS actually persisted.
    void checkKeyStatus();
  };

  const deleteKey = async () => {
    await invoke(IpcCommands.DELETE_API_KEY, { provider });
    setHasKey(false);
    setError(null);
    void checkKeyStatus();
  };

  return { hasKey, isChecking, error, saveKey, deleteKey, refresh: checkKeyStatus };
}
