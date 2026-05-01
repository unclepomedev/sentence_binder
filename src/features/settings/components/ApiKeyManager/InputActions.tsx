import { Save, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";

interface InputActionsProps {
  inputValue: string;
  onInputChange: (value: string) => void;
  onSave: () => void;
  onDelete: () => void;
  canDelete: boolean;
}

export function InputActions({
  inputValue,
  onInputChange,
  onSave,
  onDelete,
  canDelete,
}: InputActionsProps) {
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
        className="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-primary"
      />
      <div className="flex gap-2">
        <Button onClick={onSave} disabled={!inputValue.trim()}>
          <Save className="h-4 w-4 mr-2" />
          Save
        </Button>
        <Button
          variant="outline"
          onClick={onDelete}
          disabled={!canDelete}
          className="hover:text-destructive hover:border-destructive transition-colors"
        >
          <Trash2 className="h-4 w-4 mr-2" />
          Delete
        </Button>
      </div>
    </div>
  );
}
