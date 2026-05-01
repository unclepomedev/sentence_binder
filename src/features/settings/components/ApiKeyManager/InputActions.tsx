import { Loader2, Save, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

interface InputActionsProps {
  inputValue: string;
  onInputChange: (value: string) => void;
  onSave: () => void;
  onDelete: () => void;
  canDelete: boolean;
  isSaving: boolean;
  isDeleting: boolean;
}

export function InputActions({
  inputValue,
  onInputChange,
  onSave,
  onDelete,
  canDelete,
  isSaving,
  isDeleting,
}: InputActionsProps) {
  const busy = isSaving || isDeleting;
  return (
    <div className="flex flex-col gap-3 sm:flex-row">
      <Input
        type="password"
        value={inputValue}
        onChange={(e) => onInputChange(e.target.value)}
        placeholder="sk-..."
        aria-label="API key"
        autoComplete="new-password"
        spellCheck={false}
        autoCorrect="off"
        autoCapitalize="none"
        data-1p-ignore
        data-lpignore="true"
        data-form-type="other"
        disabled={busy}
        className="flex-1 bg-background"
      />
      <div className="flex gap-2">
        <Button onClick={onSave} disabled={!inputValue.trim() || busy}>
          {isSaving ? <Loader2 className="animate-spin" /> : <Save />}
          {isSaving ? "Saving..." : "Save"}
        </Button>
        <Button
          variant="outline"
          onClick={onDelete}
          disabled={!canDelete || busy}
          className="transition-colors"
        >
          {isDeleting ? <Loader2 className="animate-spin" /> : <Trash2 />}
          {isDeleting ? "Deleting..." : "Delete"}
        </Button>
      </div>
    </div>
  );
}
