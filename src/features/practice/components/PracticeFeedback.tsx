import { BookOpen, Lightbulb, type LucideIcon, Sparkles } from "lucide-react";
import type { ReactNode } from "react";
import type { ProofreadFeedback } from "@/types";

interface PracticeFeedbackProps {
  feedback?: ProofreadFeedback;
}

export function PracticeFeedback({ feedback }: PracticeFeedbackProps) {
  if (!feedback) return null;

  return (
    <div className="flex flex-col gap-4">
      <FeedbackBox>{feedback.feedback}</FeedbackBox>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        <StatCard icon={Lightbulb} label="Key Expression">
          <p className="text-sm font-medium text-foreground">{feedback.key_expression}</p>
        </StatCard>

        <StatCard icon={BookOpen} label="Example Usage">
          <p className="text-sm italic text-muted-foreground">"{feedback.example}"</p>
        </StatCard>
      </div>
    </div>
  );
}

function FeedbackBox({ children }: { children: ReactNode }) {
  return (
    <div className="bg-primary/5 border border-primary/20 rounded-lg p-4 space-y-2">
      <div className="flex items-center gap-2 text-primary text-sm font-semibold">
        <Sparkles className="h-4 w-4" /> AI Feedback
      </div>
      <p className="text-sm leading-relaxed text-foreground">{children}</p>
    </div>
  );
}

function StatCard({
  icon: Icon,
  label,
  children,
}: {
  icon: LucideIcon;
  label: string;
  children: ReactNode;
}) {
  return (
    <div className="bg-muted/30 border border-muted rounded-lg p-3 space-y-1.5">
      <div className="flex items-center gap-2 text-muted-foreground text-xs font-semibold uppercase tracking-wider">
        <Icon className="h-3.5 w-3.5" /> {label}
      </div>
      {children}
    </div>
  );
}
