import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { toast } from "sonner";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useSentences } from "@/hooks/useSentences";
import { IpcCommands } from "@/types/ipc";
import { SentenceCard } from "./SentenceCard";

export function LibraryView() {
  const { sentences, isLoading, error } = useSentences();

  // Track which sentence is currently being spoken by its ID:
  //  if any audio is currently playing, block the click
  const [playingId, setPlayingId] = useState<string | null>(null);

  const handleToggleAudio = async (id: string, text: string) => {
    if (playingId === id) {
      setPlayingId(null);
      await invoke(IpcCommands.STOP_AUDIO);
      return;
    }
    if (playingId) return;
    setPlayingId(id);

    try {
      await invoke(IpcCommands.PLAY_PRONUNCIATION, { text });
    } catch (err) {
      // If the error happens while playingId is null, it means the user
      // intentionally stopped it, so we suppress the error toast.
      // If playingId is still set, it was a genuine failure.
      if (playingId === id) {
        console.error(err);
        toast.error("Failed to play audio");
      }
    } finally {
      // Clean up state just in case it naturally finished
      setPlayingId((prev) => (prev === id ? null : prev));
    }
  };

  return (
    <div className="flex flex-col h-full gap-4 overflow-hidden">
      <header className="flex-none">
        <h1 className="text-2xl font-bold tracking-tight">Sentence Library</h1>
      </header>

      <main className="flex-1 flex flex-col min-h-0 overflow-hidden">
        {isLoading && <p className="text-sm text-muted-foreground flex-none">Loading...</p>}
        {error && <p className="text-sm text-destructive flex-none">Failed to load data.</p>}

        <ScrollArea className="flex-1 rounded-md border p-4">
          <div className="flex flex-col gap-4">
            {sentences.length === 0 && !isLoading && !error && (
              <p className="text-sm text-muted-foreground text-center py-10">
                No sentences saved yet.
              </p>
            )}

            {sentences.map((item) => (
              <SentenceCard
                key={item.id}
                item={item}
                isPlaying={playingId === item.id}
                isLocked={playingId !== null}
                onTogglePlay={() => handleToggleAudio(item.id, item.original_text)}
              />
            ))}
          </div>
        </ScrollArea>
      </main>
    </div>
  );
}
