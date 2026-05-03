import type { Sentence } from "@/types";

interface PracticeTargetProps {
  sentence: Sentence;
}

export function PracticeTarget({ sentence }: PracticeTargetProps) {
  return (
    <div className="space-y-2">
      <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">
        Target Meaning
      </h2>
      <p className="text-xl font-medium leading-relaxed">{sentence.translated_text}</p>
      {sentence.source_context && (
        <p className="text-xs text-muted-foreground bg-muted/30 inline-block px-2 py-1 rounded">
          Context: {sentence.source_context}
        </p>
      )}
    </div>
  );
}
