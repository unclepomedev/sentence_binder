import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { IpcCommands } from "@/types/ipc";

// Maximum time to wait for the keychain status check before treating the
// IPC call as hung. Without this, a Tauri/IPC deadlock or stalled keychain
// prompt would leave the UI stuck on "Checking..." with no Retry button.
const HAS_API_KEY_TIMEOUT_MS = 8000;

// After this much time elapses while still "checking", we consider the check
// long-running enough that the user should be offered a manual retry, even
// though `isChecking` is still true. This guards against true JS-thread
// stalls/deadlocks where `withTimeout`'s setTimeout itself can't fire.
const CHECKING_STUCK_THRESHOLD_MS = 10000;

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
  const [isStuck, setIsStuck] = useState<boolean>(false);

  // Monotonic request id: when the user manually retries we increment this
  // and ignore late results from any earlier in-flight check, so a hung
  // previous call cannot overwrite the newer one's outcome.
  const requestIdRef = useRef(0);

  const checkKeyStatus = useCallback(async () => {
    const myRequestId = ++requestIdRef.current;
    setIsChecking(true);
    setIsStuck(false);

    // Surface a manual-retry path even if the underlying invoke (and the
    // withTimeout setTimeout itself) never resolves — e.g. a JS-thread stall
    // where timers don't fire. We schedule this independently and only act
    // if we are still the latest in-flight request.
    const stuckTimer = setTimeout(() => {
      if (requestIdRef.current === myRequestId) {
        setIsStuck(true);
      }
    }, CHECKING_STUCK_THRESHOLD_MS);

    try {
      const exists = await withTimeout(
        invoke<boolean>(IpcCommands.HAS_API_KEY, { provider }),
        HAS_API_KEY_TIMEOUT_MS,
        "Key status check",
      );
      if (requestIdRef.current !== myRequestId) return;
      setHasKey(exists);
      setError(null);
    } catch (err) {
      if (requestIdRef.current !== myRequestId) return;
      console.error("[useCredentials] Failed to check key status:", err);
      if (err instanceof Error) {
        setError(err);
      } else {
        setError(new Error(String(err)));
      }
    } finally {
      clearTimeout(stuckTimer);
      if (requestIdRef.current === myRequestId) {
        setIsChecking(false);
        setIsStuck(false);
      }
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

  return { hasKey, isChecking, isStuck, error, saveKey, deleteKey, refresh: checkKeyStatus };
}
