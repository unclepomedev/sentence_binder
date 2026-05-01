import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { IpcCommands } from "@/types/ipc";

// Max wait for IPC response before failing (prevents UI hanging on silent IPC drops).
const HAS_API_KEY_TIMEOUT_MS = 8000;

// Failsafe threshold: If `isChecking` remains true past this time, surface a
// manual Retry path even before the IPC timeout fires. MUST be shorter than
// `HAS_API_KEY_TIMEOUT_MS`, otherwise `withTimeout` always rejects first and
// `isStuck` is unreachable in practice.
const CHECKING_STUCK_THRESHOLD_MS = 4000;

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
  const stuckTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  // Prevents late-resolving IPC calls or timers from triggering state updates after unmount.
  const isMountedRef = useRef(true);

  const checkKeyStatus = useCallback(async () => {
    const myRequestId = ++requestIdRef.current;
    if (!isMountedRef.current) return;
    setIsChecking(true);
    setIsStuck(false);
    setError(null);
    // Reset hasKey so the UI accurately reflects "Checking…" instead of
    // continuing to display the previously-known state during revalidation.
    setHasKey(null);

    // Failsafe timer: Unlocks the 'Retry' button if the timeout promise itself fails to fire.
    if (stuckTimerRef.current !== null) {
      clearTimeout(stuckTimerRef.current);
    }
    stuckTimerRef.current = setTimeout(() => {
      if (!isMountedRef.current) return;
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
      if (!isMountedRef.current) return;
      if (requestIdRef.current !== myRequestId) return;
      setHasKey(exists);
      setError(null);
    } catch (err) {
      if (!isMountedRef.current) return;
      if (requestIdRef.current !== myRequestId) return;
      console.error("[useCredentials] Failed to check key status:", err);
      if (err instanceof Error) {
        setError(err);
      } else {
        setError(new Error(String(err)));
      }
    } finally {
      if (stuckTimerRef.current !== null) {
        clearTimeout(stuckTimerRef.current);
        stuckTimerRef.current = null;
      }
      if (isMountedRef.current && requestIdRef.current === myRequestId) {
        setIsChecking(false);
        // Intentionally do NOT clear isStuck here: if a stall was detected,
        // keep the signal visible until the next check starts
      }
    }
  }, [provider]);

  useEffect(() => {
    isMountedRef.current = true;
    void checkKeyStatus();
    return () => {
      // Cleanup: mark unmounted and clear timers to prevent memory leaks or zombie state updates.
      isMountedRef.current = false;
      requestIdRef.current++;
      if (stuckTimerRef.current !== null) {
        clearTimeout(stuckTimerRef.current);
        stuckTimerRef.current = null;
      }
    };
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
