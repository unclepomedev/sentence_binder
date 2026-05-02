import { Textarea } from "@/components/ui/textarea";

interface PracticeInputProps {
  attempt: string;
  isProofreading: boolean;
  onChange: (value: string) => void;
}

export function PracticeInput({ attempt, isProofreading, onChange }: PracticeInputProps) {
  return (
    <div className="flex flex-col gap-2">
      <Textarea
        placeholder="Type your translation here..."
        className="min-h-37.5 resize-none text-base p-4 bg-background border-muted/50 focus-visible:ring-1"
        value={attempt}
        onChange={(e) => onChange(e.target.value)}
        disabled={isProofreading}
      />
    </div>
  );
}
