import { useState } from "react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useDebounce } from "@/hooks/useDebounce";
import { LibrarySearch } from "./components/LibrarySearch";
import { SentenceCard } from "./components/SentenceCard";
import { useDeleteSentence } from "./hooks/useDeleteSentence";
import { usePronunciation } from "./hooks/usePronunciation";
import { useSentences } from "./hooks/useSentences";
import { useUpdateTranslation } from "./hooks/useUpdateTranslation";

const SEARCH_DEBOUNCE_DELAY_MS = 300;

export function LibraryView() {
  const [searchTerm, setSearchTerm] = useState("");
  const debouncedSearchTerm = useDebounce(searchTerm, SEARCH_DEBOUNCE_DELAY_MS);
  const normalizedQuery = debouncedSearchTerm.trim();
  const { sentences, isLoading, error } = useSentences(normalizedQuery);
  const { playingId, toggleAudio, stopAudio } = usePronunciation();
  const { updateTranslation } = useUpdateTranslation();
  const { deleteSentence } = useDeleteSentence();

  return (
    <div className="flex flex-col h-full gap-4 overflow-hidden">
      <header className="flex-none flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <h1 className="text-2xl font-bold tracking-tight">Sentence Library</h1>
        <LibrarySearch value={searchTerm} onChange={setSearchTerm} />
      </header>

      <main className="flex-1 min-h-0 overflow-hidden">
        <ScrollArea className="h-full rounded-md border bg-card/50">
          <div className="flex flex-col gap-4 p-4">
            {isLoading && <p className="text-sm text-muted-foreground text-center">Loading...</p>}
            {error && <p className="text-sm text-destructive text-center">Failed to load data.</p>}

            {sentences.length === 0 && !isLoading && !error && (
              <div className="flex flex-col items-center justify-center py-10 text-center">
                <p className="text-sm text-muted-foreground">
                  {normalizedQuery
                    ? `No results found for "${normalizedQuery}"`
                    : "No sentences saved yet."}
                </p>
                {normalizedQuery && (
                  <Button variant="link" onClick={() => setSearchTerm("")} className="mt-2 text-sm">
                    Clear search
                  </Button>
                )}
              </div>
            )}

            {sentences.map((item) => (
              <SentenceCard
                key={item.id}
                item={item}
                searchQuery={normalizedQuery}
                isPlaying={playingId === item.id}
                isLocked={playingId !== null}
                onTogglePlay={() => toggleAudio(item.id, item.original_text)}
                onSaveEdit={updateTranslation}
                onDelete={deleteSentence}
                onStopAudio={stopAudio}
                onTagClick={(tag) => setSearchTerm(`tag:${tag}`)}
              />
            ))}
          </div>
        </ScrollArea>
      </main>
    </div>
  );
}
