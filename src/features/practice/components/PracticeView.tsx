import { Bot, Loader2 } from "lucide-react";
import { useEffect, useState } from "react";
import { useSentences } from "@/features/library/hooks/useSentences";
import type { Sentence } from "@/types";
import { useProofread } from "../hooks/useProofread";
import { PracticeCard } from "./PracticeCard";

export function PracticeView() {
  const { sentences, isLoading } = useSentences("");
  const { proofread, feedback, isProofreading, reset } = useProofread();

  const [currentSentence, setCurrentSentence] = useState<Sentence | null>(null);
  const [attempt, setAttempt] = useState("");
  const [showOriginal, setShowOriginal] = useState(false);

  // Practice requires a non-empty translation as the target. Filter out sentences
  // whose translation is missing/blank (e.g., translation failed or user cleared it).
  const practiceCandidates = sentences.filter((s) => s.translated_text.trim().length > 0);

  // Initialize the first sentence when data loads
  useEffect(() => {
    if (practiceCandidates.length > 0 && !currentSentence) {
      const randomIndex = Math.floor(Math.random() * practiceCandidates.length);
      setCurrentSentence(practiceCandidates[randomIndex]);
    }
  }, [practiceCandidates, currentSentence]);

  const handleNext = () => {
    if (practiceCandidates.length === 0) return;
    const randomIndex = Math.floor(Math.random() * practiceCandidates.length);
    setCurrentSentence(practiceCandidates[randomIndex]);
    setAttempt("");
    setShowOriginal(false);
    reset();
  };

  const handleProofread = async () => {
    if (!currentSentence || !attempt.trim()) return;
    try {
      await proofread({
        originalText: currentSentence.original_text,
        translatedText: currentSentence.translated_text,
        userAttempt: attempt,
      });
      setShowOriginal(true);
    } catch {
      // Error is surfaced via the mutation's onError toast; swallow here to
      // avoid unhandled promise rejections on IPC/LLM failures.
    }
  };

  if (isLoading) {
    return (
      <div className="h-full flex items-center justify-center text-muted-foreground">
        <Loader2 className="animate-spin h-6 w-6 mr-2" /> Loading practice data...
      </div>
    );
  }

  if (sentences.length === 0) {
    return (
      <div className="h-full flex flex-col items-center justify-center text-muted-foreground">
        <Bot className="h-12 w-12 mb-4 opacity-20" />
        <p>Your library is empty. Add some sentences first!</p>
      </div>
    );
  }

  if (practiceCandidates.length === 0) {
    return (
      <div className="h-full flex flex-col items-center justify-center text-muted-foreground">
        <Bot className="h-12 w-12 mb-4 opacity-20" />
        <p>No translatable sentences yet (missing translations).</p>
      </div>
    );
  }

  if (!currentSentence) return null;

  return (
    <div className="h-full flex flex-col max-w-3xl mx-auto py-6 overflow-hidden">
      <div className="mb-8 shrink-0">
        <h1 className="text-2xl font-bold tracking-tight">Practice Mode</h1>
        <p className="text-muted-foreground text-sm">
          Translate the target meaning back into the original language.
        </p>
      </div>

      <PracticeCard
        sentence={currentSentence}
        attempt={attempt}
        feedback={feedback}
        showOriginal={showOriginal}
        isProofreading={isProofreading}
        onAttemptChange={setAttempt}
        onToggleOriginal={() => setShowOriginal((prev) => !prev)}
        onProofread={handleProofread}
        onNext={handleNext}
      />
    </div>
  );
}
