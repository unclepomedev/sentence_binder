import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { Sentence } from "@/types";
import { IpcCommands } from "@/types/ipc";
import { sentencesQueryKey } from "./sentencesQueryKey";

export function useSentences(searchQuery: string = "") {
  const normalized = searchQuery.trim();
  const query = useQuery({
    queryKey: sentencesQueryKey(normalized),
    queryFn: async () => {
      return await invoke<Sentence[]>(IpcCommands.GET_SENTENCES, {
        searchQuery: normalized || null,
      });
    },
  });

  return {
    sentences: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
  };
}
