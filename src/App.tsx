import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { Toaster } from "@/components/ui/sonner";
import { LibraryView } from "@/features/library/LibraryView";
import { SettingsTest } from "@/features/settings/SettingsTest";
import { useCapture } from "@/hooks/useCapture";
import type { Sentence } from "@/types";
import { IpcCommands } from "@/types/ipc";

function App() {
  const queryClient = useQueryClient();

  useCapture((capturedText) => {
    // TODO: make a constant
    // Truncate the text for the toast preview so it doesn't take up the whole screen
    const previewText = capturedText.length > 30 ? `${capturedText.slice(0, 30)}...` : capturedText;

    const savePromise = invoke<Sentence>(IpcCommands.SAVE_SENTENCE, {
      originalText: capturedText,
      // TODO: pass null for now, unless have a way to grab the active window's title/URL
      sourceContext: null,
    });

    toast.promise(savePromise, {
      loading: `Translating: "${previewText}"`,
      success: (newSentence) => {
        queryClient.setQueryData(["sentences"], (old: Sentence[] | undefined) => {
          return old ? [newSentence, ...old] : [newSentence];
        });

        return "Saved and translated successfully!";
      },
      error: (err) => `Failed to process: ${err}`,
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
