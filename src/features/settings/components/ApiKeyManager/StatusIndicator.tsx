import { AlertCircle } from "lucide-react";

interface StatusIndicatorProps {
  error: Error | null;
  hasKey: boolean | null;
  label: string;
}

export function StatusIndicator({ error, hasKey, label }: StatusIndicatorProps) {
  const dotColor = error
    ? "bg-yellow-500"
    : hasKey === null
      ? "bg-gray-400 animate-pulse"
      : hasKey
        ? "bg-green-500"
        : "bg-destructive";

  return (
    <div className="flex items-center gap-3 p-3 rounded-md bg-muted/50 border border-border/50">
      <div className={`size-2.5 rounded-full shadow-sm ${dotColor}`} />
      <span className="text-sm font-medium flex items-center gap-2 text-foreground">
        {error ? (
          <>
            <AlertCircle className="h-4 w-4 text-yellow-500" />
            Keychain error. Try saving a key to reset.
          </>
        ) : hasKey === null ? (
          "Checking..."
        ) : hasKey ? (
          `${label} Key is active`
        ) : (
          `No ${label} Key found`
        )}
      </span>
    </div>
  );
}
