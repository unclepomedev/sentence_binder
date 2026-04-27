import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { Sentence } from "@/types";

export function useSentences() {
  const query = useQuery({
    queryKey: ["sentences"],
    queryFn: async () => {
      return await invoke<Sentence[]>("get_sentences");
    },
  });

  return {
    sentences: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
  };
}
