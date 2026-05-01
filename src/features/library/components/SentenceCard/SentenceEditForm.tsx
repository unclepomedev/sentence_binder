import { Check, Loader2, X } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";

interface SentenceEditFormProps {
  initialText: string;
  initialContext: string | null;
  onSave: (newText: string, newContext: string | null) => Promise<void>;
  onCancel: () => void;
}

export function SentenceEditForm({
  initialText,
  initialContext,
  onSave,
  onCancel,
}: SentenceEditFormProps) {
  const [draftText, setDraftText] = useState(initialText);
  const [draftContext, setDraftContext] = useState(initialContext || "");
  const [isSaving, setIsSaving] = useState(false);

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await onSave(draftText.trim(), draftContext.trim() || null);
    } catch {
      // Error is handled/toasted upstream; just allow the user to retry.
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="flex flex-col gap-3 mt-2">
      <div className="space-y-2">
        <Textarea
          value={draftText}
          onChange={(e) => setDraftText(e.target.value)}
          placeholder="Type your translation here..."
          disabled={isSaving}
          className="resize-y bg-background"
        />
        <Input
          type="text"
          className="text-[11px] text-muted-foreground bg-muted/30 focus-visible:bg-background placeholder:text-muted-foreground/50"
          value={draftContext}
          onChange={(e) => setDraftContext(e.target.value)}
          placeholder="Source context (e.g. App Name - URL)"
          disabled={isSaving}
        />
      </div>
      <div className="flex justify-end gap-2">
        <Button variant="ghost" size="sm" onClick={onCancel} disabled={isSaving}>
          <X /> Cancel
        </Button>
        <Button size="sm" onClick={handleSave} disabled={isSaving}>
          {isSaving ? <Loader2 className="animate-spin" /> : <Check />}
          Save
        </Button>
      </div>
    </div>
  );
}
