import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { Sentence } from "@/types";
import { IpcCommands } from "@/types/ipc";

export function useSentences() {
  const query = useQuery({
    queryKey: ["sentences"],
    queryFn: async () => {
      return await invoke<Sentence[]>(IpcCommands.GET_SENTENCES);
    },
  });

  return {
    sentences: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
  };
}
