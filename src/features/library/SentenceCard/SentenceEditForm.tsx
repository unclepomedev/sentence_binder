import { Check, Loader2, X } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";

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
        <textarea
          className="w-full min-h-20 p-2 text-sm rounded-md border bg-background resize-y focus:outline-none focus:ring-1 focus:ring-primary"
          value={draftText}
          onChange={(e) => setDraftText(e.target.value)}
          placeholder="Type your translation here..."
          disabled={isSaving}
        />
        <input
          type="text"
          className="w-full p-2 text-[11px] rounded-md border bg-muted/30 focus:bg-background focus:outline-none focus:ring-1 focus:ring-primary text-muted-foreground placeholder:text-muted-foreground/50"
          value={draftContext}
          onChange={(e) => setDraftContext(e.target.value)}
          placeholder="Source context (e.g. App Name - URL)"
          disabled={isSaving}
        />
      </div>
      <div className="flex justify-end gap-2">
        <Button variant="ghost" size="sm" onClick={onCancel} disabled={isSaving}>
          <X className="h-4 w-4 mr-1" /> Cancel
        </Button>
        <Button size="sm" onClick={handleSave} disabled={isSaving}>
          {isSaving ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Check className="h-4 w-4 mr-1" />
          )}
          Save
        </Button>
      </div>
    </div>
  );
}
