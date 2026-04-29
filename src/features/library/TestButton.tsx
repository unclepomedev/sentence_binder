import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";

export function TestButton() {
  const [result, setResult] = useState<string>("");
  const [isLoading, setIsLoading] = useState(false);

  const testTranslation = async () => {
    setIsLoading(true);
    setResult("");

    try {
      const text = "The architecture is messy, but at least the data is flowing!";
      toast.info("Sending translation request to MLX...");

      const res = await invoke<string>("translate_text", { text });
      setResult(res);
      toast.success("Translation successful!");
    } catch (error) {
      console.error(error);
      setResult(`Error: ${error}`);
      toast.error("Translation failed");
    } finally {
      setIsLoading(false);
    }
  };

  const testUsage = async () => {
    setIsLoading(true);
    setResult("");

    try {
      const expression = "data is flowing";
      const context = "The architecture is messy, but at least the data is flowing!";
      toast.info("Sending usage extraction request to MLX...");

      const res = await invoke<string>("extract_usage", { expression, context });
      setResult(res);
      toast.success("Usage extraction successful!");
    } catch (error) {
      console.error(error);
      setResult(`Error: ${error}`);
      toast.error("Usage extraction failed");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="p-4 border rounded-xl bg-card text-card-foreground flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h3 className="font-semibold">Local LLM (MLX) Connection Test</h3>
      </div>

      <div className="flex gap-3">
        <Button onClick={testTranslation} disabled={isLoading}>
          {isLoading ? "Processing..." : "Test Translation"}
        </Button>
        <Button variant="secondary" onClick={testUsage} disabled={isLoading}>
          {isLoading ? "Processing..." : "Test Usage"}
        </Button>
      </div>

      {result && (
        <div className="p-4 bg-muted rounded-lg text-sm whitespace-pre-wrap font-mono">
          {result}
        </div>
      )}
    </div>
  );
}
