import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useSentences } from "@/hooks/useSentences";

export function LibraryView() {
  const { sentences, isLoading, error } = useSentences();

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
