import { KeyRound } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ApiKeyManager } from "./ApiKeyManager";

export function WebLLMSettings() {
  return (
    <Card className="bg-card shadow-sm max-w-2xl">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-lg">
          <KeyRound className="h-5 w-5 text-muted-foreground" />
          Web LLM Configuration
        </CardTitle>
        <CardDescription>
          Manage API keys for external translation and generation models. Keys are stored securely
          in your system keychain.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <ApiKeyManager providerId="openai" label="OpenAI API" />
      </CardContent>
    </Card>
  );
}
