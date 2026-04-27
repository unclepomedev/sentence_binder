import { Button } from "@/components/ui/button.tsx";
import { Card, CardContent, CardHeader } from "@/components/ui/card.tsx";
import { ScrollArea } from "@/components/ui/scroll-area.tsx";
import { useSentences } from "@/hooks/useSentences.ts";

export function LibraryView() {
  const { sentences, isLoading, error, addTestSentence } = useSentences();

  return (
    <div className="flex flex-col h-full gap-4">
      <header className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Sentence Library</h1>
        <Button onClick={() => addTestSentence.mutate()} disabled={addTestSentence.isPending}>
          {addTestSentence.isPending ? "Saving..." : "Add Test Data"}
        </Button>
      </header>

      <main className="flex-1 overflow-hidden">
        {isLoading && <p className="text-sm text-muted-foreground">Loading...</p>}
        {error && <p className="text-sm text-destructive">Failed to load data.</p>}

        <ScrollArea className="h-full rounded-md border p-4">
          <div className="flex flex-col gap-4">
            {sentences.length === 0 && !isLoading && (
              <p className="text-sm text-muted-foreground text-center py-10">
                No sentences saved yet.
              </p>
            )}
            {sentences.map((item) => (
              <Card key={item.id} className="bg-card shadow-sm">
                <CardHeader className="pb-2">
                  <p className="text-base font-medium leading-tight">{item.original_text}</p>
                </CardHeader>
                <CardContent>
                  <p className="text-sm text-muted-foreground">{item.translated_text}</p>
                  {item.source_context && (
                    <p className="text-[10px] text-muted-foreground/60 mt-3 text-right uppercase tracking-wider">
                      {item.source_context}
                    </p>
                  )}
                </CardContent>
              </Card>
            ))}
          </div>
        </ScrollArea>
      </main>
    </div>
  );
}
