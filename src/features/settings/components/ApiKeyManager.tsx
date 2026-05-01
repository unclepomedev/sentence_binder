import { Save, Trash2 } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { useCredentials } from "@/hooks/useCredentials";

interface ApiKeyManagerProps {
  providerId: string;
  label: string;
}

export function ApiKeyManager({ providerId, label }: ApiKeyManagerProps) {
  const { hasKey, saveKey, deleteKey } = useCredentials(providerId);
  const [inputValue, setInputValue] = useState("");

  const handleSave = async () => {
    const trimmed = inputValue.trim();
    if (!trimmed) return;
    try {
      await saveKey(trimmed);
      toast.success(`${label} Key saved securely.`);
      setInputValue("");
    } catch (error) {
      toast.error(`Failed to save: ${error}`);
    }
  };

  const handleDelete = async () => {
    try {
      await deleteKey();
      toast.info(`${label} Key removed.`);
    } catch (error) {
      toast.error(`Failed to delete: ${error}`);
    }
  };

  return (
    <div className="space-y-4">
      {/* Status Indicator */}
      <div className="flex items-center gap-3 p-3 rounded-md bg-muted/50 border border-border/50">
        <div
          className={`size-2.5 rounded-full shadow-sm ${
            hasKey === null
              ? "bg-gray-400 animate-pulse"
              : hasKey
                ? "bg-green-500"
                : "bg-destructive"
          }`}
        />
        <span className="text-sm font-medium text-foreground">
          {hasKey === null
            ? "Checking..."
            : hasKey
              ? `${label} Key is active`
              : `No ${label} Key found`}
        </span>
      </div>

      {/* Input & Actions */}
      <div className="flex flex-col gap-3 sm:flex-row">
        <input
          type="password"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          placeholder="sk-..."
          className="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-primary"
        />
        <div className="flex gap-2">
          <Button onClick={handleSave} disabled={!inputValue.trim()}>
            <Save className="h-4 w-4 mr-2" />
            Save
          </Button>
          <Button
            variant="outline"
            onClick={handleDelete}
            disabled={!hasKey}
            className="hover:text-destructive hover:border-destructive transition-colors"
          >
            <Trash2 className="h-4 w-4 mr-2" />
            Delete
          </Button>
        </div>
      </div>
    </div>
  );
}
