import { useMutation } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { IpcCommands } from "@/types/ipc";

type ProofreadPayload = {
  originalText: string;
  translatedText: string;
  userAttempt: string;
};

export function useProofread() {
  const mutation = useMutation({
    mutationFn: async (payload: ProofreadPayload) => {
      return await invoke<string>(IpcCommands.PROOFREAD_SENTENCE, payload);
    },
    onError: (err) => {
      console.error(err);
      const message = err instanceof Error ? err.message : String(err);
      toast.error(`Proofread failed: ${message}`);
    },
  });

  return {
    proofread: mutation.mutateAsync,
    feedback: mutation.data,
    isProofreading: mutation.isPending,
    reset: mutation.reset,
  };
}
