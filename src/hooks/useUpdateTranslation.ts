import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type { Sentence } from "@/types";
import { IpcCommands } from "@/types/ipc";

export function useUpdateTranslation() {
  const queryClient = useQueryClient();

  const updateTranslation = async (id: string, newText: string, newContext: string | null) => {
    try {
      await invoke(IpcCommands.UPDATE_SENTENCE_TRANSLATION, {
        id,
        newTranslation: newText,
        newContext,
      });

      queryClient.setQueryData(["sentences"], (old: Sentence[] | undefined) =>
        old?.map((s) =>
          s.id === id
            ? {
                ...s,
                translated_text: newText,
                source_context: newContext || null,
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
