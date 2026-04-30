import { ScrollArea } from "@/components/ui/scroll-area";
import { useDeleteSentence } from "@/hooks/useDeleteSentence";
import { usePronunciation } from "@/hooks/usePronunciation";
import { useSentences } from "@/hooks/useSentences";
import { useUpdateTranslation } from "@/hooks/useUpdateTranslation";
import { SentenceCard } from "./SentenceCard";

export function LibraryView() {
  const { sentences, isLoading, error } = useSentences();
  const { playingId, toggleAudio } = usePronunciation();
  const { updateTranslation } = useUpdateTranslation();
  const { deleteSentence } = useDeleteSentence();

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
                onTogglePlay={() => toggleAudio(item.id, item.original_text)}
                onSaveEdit={updateTranslation}
                onDelete={deleteSentence}
              />
            ))}
          </div>
        </ScrollArea>
      </main>
    </div>
  );
}
