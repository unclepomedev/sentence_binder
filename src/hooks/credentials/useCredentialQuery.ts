import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { IpcCommands } from "@/types/ipc";
import { useIsMountedRef, withTimeout } from "./utils";

// Max wait for IPC response before failing.
const HAS_API_KEY_TIMEOUT_MS = 8000;
// Failsafe threshold: Unlocks manual retry before the IPC timeout fires.
const CHECKING_STUCK_THRESHOLD_MS = 4000;

export interface CredentialQueryResult {
  hasKey: boolean | null;
  isChecking: boolean;
  isStuck: boolean;
  error: Error | null;
  refresh: () => Promise<void>;
  /** Imperatively update query state (used by mutations for optimistic updates). */
  setHasKey: (value: boolean | null) => void;
  setError: (err: Error | null) => void;
}

export function useCredentialQuery(provider: string): CredentialQueryResult {
  const [hasKey, setHasKey] = useState<boolean | null>(null);
  const [isChecking, setIsChecking] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);
  const [isStuck, setIsStuck] = useState<boolean>(false);

  const requestIdRef = useRef(0);
  const stuckTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isMountedRef = useIsMountedRef();

  const checkKeyStatus = useCallback(async () => {
    const myRequestId = ++requestIdRef.current;
    if (!isMountedRef.current) return;
    setIsChecking(true);
    setIsStuck(false);
    setError(null);
    setHasKey(null);

    if (stuckTimerRef.current !== null) clearTimeout(stuckTimerRef.current);

    // Shadow the timer locally so concurrent checks don't clear each other's failsafes.
    const myStuckTimer = setTimeout(() => {
      if (isMountedRef.current && requestIdRef.current === myRequestId) {
        setIsStuck(true);
      }
    }, CHECKING_STUCK_THRESHOLD_MS);
    stuckTimerRef.current = myStuckTimer;

    try {
      const exists = await withTimeout(
        invoke<boolean>(IpcCommands.HAS_API_KEY, { provider }),
        HAS_API_KEY_TIMEOUT_MS,
        "Key status check",
      );
      if (!isMountedRef.current || requestIdRef.current !== myRequestId) return;
      setHasKey(exists);
      setError(null);
    } catch (err) {
      if (!isMountedRef.current || requestIdRef.current !== myRequestId) return;
      console.error("[useCredentials] Failed to check key status:", err);
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      // Only clear the timer this specific request created.
      clearTimeout(myStuckTimer);
      if (stuckTimerRef.current === myStuckTimer) {
        stuckTimerRef.current = null;
      }
      if (isMountedRef.current && requestIdRef.current === myRequestId) {
        setIsChecking(false);
      }
    }
  }, [provider, isMountedRef]);

  useEffect(() => {
    void checkKeyStatus();
    return () => {
      requestIdRef.current++;
      if (stuckTimerRef.current !== null) clearTimeout(stuckTimerRef.current);
    };
  }, [checkKeyStatus]);

  return { hasKey, isChecking, isStuck, error, refresh: checkKeyStatus, setHasKey, setError };
}
