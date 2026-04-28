import { useState } from "react";
import { toast } from "sonner";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { useCredentials } from "@/hooks/useCredentials";

export function SettingsTest() {
  const { hasKey, saveKey, deleteKey } = useCredentials("openai");
  const [inputValue, setInputValue] = useState("");

  const handleSave = async () => {
    const trimmed = inputValue.trim();
    if (!trimmed) return;
    try {
      await saveKey(trimmed);
      toast.success("API Key securely saved to macOS Keychain!");
      setInputValue("");
    } catch (error) {
      toast.error(`Failed to save: ${error}`);
    }
  };

  const handleDelete = async () => {
    try {
      await deleteKey();
      toast.info("API Key deleted from Keychain.");
    } catch (error) {
      toast.error(`Failed to delete: ${error}`);
    }
  };

  return (
    <Card className="bg-card shadow-sm mt-6">
      <CardHeader className="pb-2">
        <h2 className="text-lg font-medium">Keychain Test (OpenAI)</h2>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center gap-3">
          <div
            className={`size-3 rounded-full ${
              hasKey === null ? "bg-gray-300" : hasKey ? "bg-green-500" : "bg-red-500"
            }`}
          />
          <span className="text-sm text-muted-foreground">
            Status: {hasKey === null ? "Loading..." : hasKey ? "Key Found" : "No Key"}
          </span>
        </div>

        <div className="flex gap-2">
          <input
            type="password"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            placeholder="sk-..."
            className="flex-1 rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm"
          />
          <button
            type="button"
            onClick={handleSave}
            className="rounded-md bg-primary text-primary-foreground px-4 py-1 text-sm shadow hover:bg-primary/90"
          >
            Save
          </button>
          <button
            type="button"
            onClick={handleDelete}
            disabled={!hasKey}
            className="rounded-md border border-input bg-background px-4 py-1 text-sm shadow-sm hover:bg-accent disabled:opacity-50"
          >
            Delete
          </button>
        </div>
      </CardContent>
    </Card>
  );
}
