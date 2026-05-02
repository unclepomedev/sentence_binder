import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type { Sentence } from "@/types";
import { IpcCommands } from "@/types/ipc";
import { SENTENCES_QUERY_KEY_ROOT } from "./sentencesQueryKey";

export function useDeleteSentence() {
  const queryClient = useQueryClient();

  const deleteSentence = async (id: string) => {
    try {
      await invoke(IpcCommands.DELETE_SENTENCE, { id });

      queryClient.setQueriesData<Sentence[]>({ queryKey: [SENTENCES_QUERY_KEY_ROOT] }, (old) =>
        old?.filter((s) => s.id !== id),
      );

      toast.success("Sentence deleted");
    } catch (err) {
      console.error(err);
      toast.error("Failed to delete sentence");
      throw err;
    }
  };

  return { deleteSentence };
}
