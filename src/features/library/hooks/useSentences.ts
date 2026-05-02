import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { Sentence } from "@/types";
import { IpcCommands } from "@/types/ipc";

export function useSentences(searchQuery: string = "") {
  const query = useQuery({
    queryKey: ["sentences", searchQuery],
    queryFn: async () => {
      return await invoke<Sentence[]>(IpcCommands.GET_SENTENCES, {
        searchQuery: searchQuery.trim() || null,
      });
    },
  });

  return {
    sentences: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
  };
}
