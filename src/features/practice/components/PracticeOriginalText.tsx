interface PracticeOriginalTextProps {
  originalText: string;
  isVisible: boolean;
}

export function PracticeOriginalText({ originalText, isVisible }: PracticeOriginalTextProps) {
  if (!isVisible) return null;

  return (
    <div className="bg-muted/30 border border-muted rounded-lg p-4 space-y-2">
      <div className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
        Original Sentence
      </div>
      <p className="text-sm text-foreground">{originalText}</p>
    </div>
  );
}
