import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type { Sentence } from "@/types";
import { IpcCommands } from "@/types/ipc";
import { SENTENCES_QUERY_KEY_ROOT } from "./sentencesQueryKey";

export function useUpdateTranslation() {
  const queryClient = useQueryClient();

  const updateTranslation = async (
    id: string,
    newText: string,
    newContext: string | null,
    tags: string[],
  ) => {
    const trimmedContext = newContext?.trim() ?? "";
    const normalizedContext = trimmedContext === "" ? null : trimmedContext;

    try {
      await invoke(IpcCommands.UPDATE_SENTENCE_TRANSLATION, {
        id,
        newTranslation: newText,
        newContext: normalizedContext,
        tags,
      });

      queryClient.setQueriesData<Sentence[]>({ queryKey: [SENTENCES_QUERY_KEY_ROOT] }, (old) =>
        old?.map((s) =>
          s.id === id
            ? {
                ...s,
                translated_text: newText,
                source_context: normalizedContext,
                tags,
              }
            : s,
        ),
      );

      toast.success("Translation updated");
    } catch (err) {
      console.error(err);
      toast.error("Failed to save translation");
      throw err;
    }
  };

  return { updateTranslation };
}
