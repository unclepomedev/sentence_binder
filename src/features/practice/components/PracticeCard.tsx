import { Card, CardContent } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { Sentence } from "@/types";
import { PracticeControls } from "./PracticeControls";
import { PracticeFeedback } from "./PracticeFeedback";
import { PracticeInput } from "./PracticeInput";
import { PracticeOriginalText } from "./PracticeOriginalText";
import { PracticeTarget } from "./PracticeTarget";

interface PracticeCardProps {
  sentence: Sentence;
  attempt: string;
  feedback?: string;
  showOriginal: boolean;
  isProofreading: boolean;
  onAttemptChange: (value: string) => void;
  onToggleOriginal: () => void;
  onProofread: () => void;
  onNext: () => void;
}

export function PracticeCard({
  sentence,
  attempt,
  feedback,
  showOriginal,
  isProofreading,
  onAttemptChange,
  onToggleOriginal,
  onProofread,
  onNext,
}: PracticeCardProps) {
  return (
    <Card className="bg-card shadow-sm border-muted flex-1 flex flex-col overflow-hidden min-h-0">
      <div className="flex-1 min-h-0 overflow-hidden">
        <ScrollArea className="h-full">
          <CardContent className="p-6 flex flex-col gap-6">
            <PracticeTarget sentence={sentence} />

            <PracticeInput
              attempt={attempt}
              isProofreading={isProofreading}
              onChange={onAttemptChange}
            />

            <PracticeFeedback feedback={feedback} />

            <PracticeOriginalText originalText={sentence.original_text} isVisible={showOriginal} />
          </CardContent>
        </ScrollArea>
      </div>

      <div className="px-6 pb-6 bg-card shrink-0">
        <PracticeControls
          attempt={attempt}
          showOriginal={showOriginal}
          isProofreading={isProofreading}
          onToggleOriginal={onToggleOriginal}
          onProofread={onProofread}
          onNext={onNext}
        />
      </div>
    </Card>
  );
}
