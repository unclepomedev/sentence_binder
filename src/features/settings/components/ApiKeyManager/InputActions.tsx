import { Loader2, Save, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";

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
      <input
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
        className="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-primary disabled:opacity-50"
      />
      <div className="flex gap-2">
        <Button onClick={onSave} disabled={!inputValue.trim() || busy}>
          {isSaving ? (
            <Loader2 className="h-4 w-4 mr-2 animate-spin" />
          ) : (
            <Save className="h-4 w-4 mr-2" />
          )}
          {isSaving ? "Saving..." : "Save"}
        </Button>
        <Button
          variant="outline"
          onClick={onDelete}
          disabled={!canDelete || busy}
          className="hover:text-destructive hover:border-destructive transition-colors"
        >
          {isDeleting ? (
            <Loader2 className="h-4 w-4 mr-2 animate-spin" />
          ) : (
            <Trash2 className="h-4 w-4 mr-2" />
          )}
          {isDeleting ? "Deleting..." : "Delete"}
        </Button>
      </div>
    </div>
  );
}
