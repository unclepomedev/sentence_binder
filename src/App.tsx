import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { toast } from "sonner";
import { Sidebar, type ViewState } from "@/components/layout/Sidebar";
import { Toaster } from "@/components/ui/sonner";
import { LibraryView } from "@/features/library/LibraryView";
import { SettingsView } from "@/features/settings/SettingsView";
import { useCapture } from "@/hooks/useCapture";
import type { Sentence } from "@/types";
import { IpcCommands } from "@/types/ipc";

const MAX_TOAST_PREVIEW_LENGTH = 30;

function App() {
  const queryClient = useQueryClient();
  const [currentView, setCurrentView] = useState<ViewState>("library");

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
    <div className="h-screen w-screen bg-background font-sans antialiased flex overflow-hidden">
      <Sidebar currentView={currentView} onViewChange={setCurrentView} />

      <div className="flex-1 min-w-0 p-6 h-full">
        {currentView === "library" ? <LibraryView /> : <SettingsView />}
      </div>

      <Toaster />
    </div>
  );
}

export default App;
