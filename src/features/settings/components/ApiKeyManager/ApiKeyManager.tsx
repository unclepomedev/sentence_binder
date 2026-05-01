import { useState } from "react";
import { toast } from "sonner";
import { useCredentials } from "@/hooks/useCredentials";
import { InputActions } from "./InputActions";
import { StatusIndicator } from "./StatusIndicator";

interface ApiKeyManagerProps {
  providerId: string;
  label: string;
}

export function ApiKeyManager({ providerId, label }: ApiKeyManagerProps) {
  const { hasKey, isChecking, isStuck, isSaving, isDeleting, saveKey, deleteKey, error, refresh } =
    useCredentials(providerId);
  const [inputValue, setInputValue] = useState("");

  const handleSave = async () => {
    const trimmed = inputValue.trim();
    if (!trimmed) return;
    try {
      await saveKey(trimmed);
      toast.success(`${label} Key saved securely.`);
      setInputValue("");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error(`Failed to save: ${message}`);
    }
  };

  const handleDelete = async () => {
    try {
      await deleteKey();
      toast.info(`${label} Key removed.`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error(`Failed to delete: ${message}`);
    }
  };

  return (
    <div className="space-y-4">
      <StatusIndicator
        error={error}
        hasKey={hasKey}
        isChecking={isChecking}
        isStuck={isStuck}
        label={label}
        onRetry={refresh}
      />

      <InputActions
        inputValue={inputValue}
        onInputChange={setInputValue}
        onSave={handleSave}
        onDelete={handleDelete}
        canDelete={!!hasKey}
        isSaving={isSaving}
        isDeleting={isDeleting}
      />
    </div>
  );
}
