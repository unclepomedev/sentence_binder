import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type { Sentence } from "@/types";

export function useSentences() {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["sentences"],
    queryFn: async () => {
      return await invoke<Sentence[]>("get_sentences");
    },
  });

  // TODO: this is temporal debugging implementation, remove it later
  const addTestSentence = useMutation({
    mutationFn: async () => {
      await invoke("save_sentence", {
        originalText: "The robust architecture ensures a seamless user experience.",
        translatedText: "堅牢なアーキテクチャが、シームレスなユーザー体験を保証します。",
        sourceContext: "Technical Specification",
      });
    },
    onSuccess: () => {
      toast.success("Sentence saved successfully.");
      queryClient.invalidateQueries({ queryKey: ["sentences"] }).then();
    },
    onError: (err) => {
      toast.error(`Failed to save: ${err}`);
    },
  });

  return {
    sentences: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
    addTestSentence,
  };
}
