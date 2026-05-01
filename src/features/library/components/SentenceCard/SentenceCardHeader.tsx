import { Square, Volume2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { CardHeader } from "@/components/ui/card";

interface SentenceCardHeaderProps {
  originalText: string;
  isPlaying: boolean;
  isLocked: boolean;
  onTogglePlay: () => void;
}

export function SentenceCardHeader({
  originalText,
  isPlaying,
  isLocked,
  onTogglePlay,
}: SentenceCardHeaderProps) {
  return (
    <CardHeader className="pb-2 flex flex-row items-start justify-between space-y-0">
      <p className="text-base font-medium leading-tight pr-4">{originalText}</p>
      <Button
        variant="ghost"
        size="icon"
        disabled={isLocked && !isPlaying}
        className="text-muted-foreground hover:text-foreground shrink-0 transition-colors"
        onClick={onTogglePlay}
      >
        {isPlaying ? <Square className="fill-current text-foreground" /> : <Volume2 />}
      </Button>
    </CardHeader>
  );
}
