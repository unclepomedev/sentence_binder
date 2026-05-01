import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { IpcCommands } from "@/types/ipc";

// TODO: temporarily passing openai as provider. change it at a good time
export function useCredentials(provider: string = "openai") {
  const [hasKey, setHasKey] = useState<boolean | null>(null);
  const [error, setError] = useState<Error | null>(null);

  const checkKeyStatus = useCallback(async () => {
    try {
      const exists = await invoke<boolean>(IpcCommands.HAS_API_KEY, { provider });
      setHasKey(exists);
      setError(null);
    } catch (err) {
      console.error("[useCredentials] Failed to check key status:", err);
      if (err instanceof Error) {
        setError(err);
      } else {
        setError(new Error(String(err)));
      }
    }
  }, [provider]);

  useEffect(() => {
    checkKeyStatus().then();
  }, [checkKeyStatus]);

  const saveKey = async (key: string) => {
    await invoke(IpcCommands.SAVE_API_KEY, { provider, key });
    setHasKey(true);
    setError(null);
  };

  const deleteKey = async () => {
    await invoke(IpcCommands.DELETE_API_KEY, { provider });
    setHasKey(false);
    setError(null);
  };

  return { hasKey, error, saveKey, deleteKey };
}
