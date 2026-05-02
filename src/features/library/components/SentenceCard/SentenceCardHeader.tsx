import { Square, Volume2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { CardHeader } from "@/components/ui/card";
import { HighlightText } from "@/components/ui/highlight-text";

interface SentenceCardHeaderProps {
  originalText: string;
  isPlaying: boolean;
  isLocked: boolean;
  searchQuery?: string;
  onTogglePlay: () => void;
}

export function SentenceCardHeader({
  originalText,
  isPlaying,
  isLocked,
  searchQuery = "",
  onTogglePlay,
}: SentenceCardHeaderProps) {
  return (
    <CardHeader className="pb-2 flex flex-row items-start justify-between space-y-0">
      <p className="text-base font-medium leading-tight pr-4">
        <HighlightText text={originalText} query={searchQuery} />
      </p>
      <Button
        variant="ghost"
        size="icon"
        disabled={isLocked && !isPlaying}
        className="text-muted-foreground hover:text-foreground shrink-0 transition-colors"
        onClick={onTogglePlay}
        aria-label={isPlaying ? "Stop pronunciation" : "Play pronunciation"}
        title={isPlaying ? "Stop pronunciation" : "Play pronunciation"}
      >
        {isPlaying ? <Square className="fill-current text-foreground" /> : <Volume2 />}
      </Button>
    </CardHeader>
  );
}
