import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { IpcCommands } from "@/types/ipc";

// TODO: temporarily passing openai as provider. change it at a good time
export function useCredentials(provider: string = "openai") {
  const [hasKey, setHasKey] = useState<boolean | null>(null);

  const checkKeyStatus = useCallback(async () => {
    try {
      await invoke<{ key: string }>(IpcCommands.GET_API_KEY, { provider });
      setHasKey(true);
    } catch {
      setHasKey(false);
    }
  }, [provider]);

  useEffect(() => {
    checkKeyStatus().then();
  }, [checkKeyStatus]);

  const saveKey = async (key: string) => {
    await invoke(IpcCommands.SAVE_API_KEY, { provider, key });
    setHasKey(true);
  };

  const deleteKey = async () => {
    await invoke(IpcCommands.DELETE_API_KEY, { provider });
    setHasKey(false);
  };

  return { hasKey, saveKey, deleteKey };
}
