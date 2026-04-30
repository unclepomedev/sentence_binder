import { Square, Volume2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import type { Sentence } from "@/types";

interface SentenceCardProps {
  item: Sentence;
  isPlaying: boolean;
  isLocked: boolean;
  onTogglePlay: () => void;
}

export function SentenceCard({ item, isPlaying, isLocked, onTogglePlay }: SentenceCardProps) {
  return (
    <Card className="bg-card shadow-sm">
      <CardHeader className="pb-2 flex flex-row items-start justify-between space-y-0">
        <p className="text-base font-medium leading-tight pr-4">{item.original_text}</p>

        <Button
          variant="ghost"
          size="icon"
          disabled={isLocked && !isPlaying}
          className="h-8 w-8 text-muted-foreground hover:text-foreground shrink-0 transition-colors"
          onClick={onTogglePlay}
        >
          {isPlaying ? (
            <Square className="h-4 w-4 fill-current text-foreground" />
          ) : (
            <Volume2 className="h-4 w-4" />
          )}
          <span className="sr-only">{isPlaying ? "Stop pronunciation" : "Play pronunciation"}</span>
        </Button>
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
  );
}
