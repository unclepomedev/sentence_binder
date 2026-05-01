import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { IpcCommands } from "@/types/ipc";
import { useIsMountedRef, withTimeout } from "./utils";

// Bounded IPC ops to prevent indefinite "Saving…"/"Deleting…" states.
const MUTATION_TIMEOUT_MS = 10000;

interface UseCredentialMutationsArgs {
  provider: string;
  refresh: () => Promise<void> | void;
  setHasKey: (value: boolean | null) => void;
  setError: (err: Error | null) => void;
}

export interface CredentialMutationsResult {
  isSaving: boolean;
  isDeleting: boolean;
  saveKey: (key: string) => Promise<void>;
  deleteKey: () => Promise<void>;
}

export function useCredentialMutations({
  provider,
  refresh,
  setHasKey,
  setError,
}: UseCredentialMutationsArgs): CredentialMutationsResult {
  const [isSaving, setIsSaving] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const isMountedRef = useIsMountedRef();

  const saveKey = async (key: string) => {
    if (!isMountedRef.current) return;
    setIsSaving(true);
    try {
      await withTimeout(
        invoke(IpcCommands.SAVE_API_KEY, { provider, key }),
        MUTATION_TIMEOUT_MS,
        "Save API key",
      );
      if (!isMountedRef.current) return;
      setHasKey(true);
      setError(null);
      // Re-verify against true OS state
      void refresh();
    } finally {
      if (isMountedRef.current) setIsSaving(false);
    }
  };

  const deleteKey = async () => {
    if (!isMountedRef.current) return;
    setIsDeleting(true);
    try {
      await withTimeout(
        invoke(IpcCommands.DELETE_API_KEY, { provider }),
        MUTATION_TIMEOUT_MS,
        "Delete API key",
      );
      if (!isMountedRef.current) return;
      setHasKey(false);
      setError(null);
      void refresh();
    } finally {
      if (isMountedRef.current) setIsDeleting(false);
    }
  };

  return { isSaving, isDeleting, saveKey, deleteKey };
}
