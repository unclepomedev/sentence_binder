import { ArrowRight, Eye, EyeOff, Loader2, Sparkles } from "lucide-react";
import { Button } from "@/components/ui/button";

interface PracticeControlsProps {
  attempt: string;
  showOriginal: boolean;
  isProofreading: boolean;
  onToggleOriginal: () => void;
  onProofread: () => void;
  onNext: () => void;
}

export function PracticeControls({
  attempt,
  showOriginal,
  isProofreading,
  onToggleOriginal,
  onProofread,
  onNext,
}: PracticeControlsProps) {
  return (
    <div className="flex items-center justify-between pt-4 border-t border-border/50">
      <Button
        variant="ghost"
        onClick={onToggleOriginal}
        disabled={isProofreading}
        className="text-muted-foreground"
      >
        {showOriginal ? (
          <>
            <EyeOff className="h-4 w-4 mr-2" />
            Hide Original
          </>
        ) : (
          <>
            <Eye className="h-4 w-4 mr-2" />
            Show Original
          </>
        )}
      </Button>

      <div className="flex gap-2">
        <Button
          variant="secondary"
          onClick={onProofread}
          disabled={isProofreading || !attempt.trim()}
        >
          {isProofreading ? (
            <Loader2 className="h-4 w-4 mr-2 animate-spin" />
          ) : (
            <Sparkles className="h-4 w-4 mr-2" />
          )}
          Proofread
        </Button>
        <Button onClick={onNext} disabled={isProofreading}>
          Next <ArrowRight className="h-4 w-4 ml-2" />
        </Button>
      </div>
    </div>
  );
}
