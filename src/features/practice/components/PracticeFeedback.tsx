import { Sparkles } from "lucide-react";

export function PracticeFeedback({ feedback }: { feedback?: string }) {
  if (!feedback) return null;

  return (
    <div className="bg-primary/5 border border-primary/20 rounded-lg p-4 space-y-2">
      <div className="flex items-center gap-2 text-primary text-sm font-semibold">
        <Sparkles className="h-4 w-4" /> AI Feedback
      </div>
      <p className="text-sm leading-relaxed text-foreground">{feedback}</p>
    </div>
  );
}
