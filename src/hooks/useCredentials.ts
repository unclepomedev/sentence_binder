import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { IpcCommands } from "@/types/ipc";

// Max wait for IPC response before failing (prevents UI hanging on silent IPC drops).
const HAS_API_KEY_TIMEOUT_MS = 8000;

// Failsafe threshold: If `isChecking` remains true past this time, we assume
// the event loop or Tauri backend is permanently stalled and allow a manual retry.
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

  // Monotonic ID to prevent race conditions. If a user clicks retry, we increment
  // this and ignore late-resolving promises from previous in-flight checks.
  const requestIdRef = useRef(0);

  const checkKeyStatus = useCallback(async () => {
    const myRequestId = ++requestIdRef.current;
    setIsChecking(true);
    setIsStuck(false);

    // Failsafe timer: Unlocks the 'Retry' button if the timeout promise itself fails to fire.
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
    // Re-verify keychain state in case the optimistic update drifts from OS reality.
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
