import { Check, Loader2, X } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";

interface SentenceEditFormProps {
  initialText: string;
  onSave: (newText: string) => Promise<void>;
  onCancel: () => void;
}

export function SentenceEditForm({ initialText, onSave, onCancel }: SentenceEditFormProps) {
  const [draft, setDraft] = useState(initialText);
  const [isSaving, setIsSaving] = useState(false);

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await onSave(draft);
    } catch {
      setIsSaving(false);
    }
  };

  return (
    <div className="flex flex-col gap-2 mt-2">
      <textarea
        className="w-full min-h-20 p-2 text-sm rounded-md border bg-background resize-y focus:outline-none focus:ring-1 focus:ring-primary"
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        placeholder="Type your translation here..."
        disabled={isSaving}
      />
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
