import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { Toaster } from "@/components/ui/sonner";
import { LibraryView } from "@/features/library/LibraryView";
import { SettingsTest } from "@/features/settings/SettingsTest";
import { useCapture } from "@/hooks/useCapture";
import type { Sentence } from "@/types";
import { IpcCommands } from "@/types/ipc";

const MAX_TOAST_PREVIEW_LENGTH = 30;

function App() {
  const queryClient = useQueryClient();

  useCapture(({ text, context }) => {
    // Truncate the text for the toast preview so it doesn't take up the whole screen
    const previewText =
      text.length > MAX_TOAST_PREVIEW_LENGTH
        ? `${text.slice(0, MAX_TOAST_PREVIEW_LENGTH)}...`
        : text;

    const savePromise = invoke<Sentence>(IpcCommands.SAVE_SENTENCE, {
      originalText: text,
      sourceContext: context,
    });

    toast.promise(savePromise, {
      loading: `Translating: "${previewText}"`,
      success: (newSentence) => {
        queryClient.setQueryData(["sentences"], (old: Sentence[] | undefined) => {
          return old ? [newSentence, ...old] : [newSentence];
        });

        return newSentence.translated_text.trim().length > 0
          ? "Saved and translated successfully!"
          : "Saved, but translation failed.";
      },
      error: (err) => `Failed to process: ${err instanceof Error ? err.message : String(err)}`,
    });
  });

  return (
    <div className="h-screen w-screen bg-background font-sans antialiased p-6 flex flex-col gap-4">
      <div className="flex-1 min-h-0">
        <LibraryView />
      </div>

      {import.meta.env.DEV && (
        <div className="flex-none flex flex-col gap-4">
          <SettingsTest />
        </div>
      )}

      <Toaster />
    </div>
  );
}

export default App;
