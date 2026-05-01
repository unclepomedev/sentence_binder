import { AlertCircle } from "lucide-react";
import { Button } from "@/components/ui/button";

interface StatusIndicatorProps {
  error: Error | null;
  hasKey: boolean | null;
  isChecking?: boolean;
  label: string;
  onRetry?: () => void | Promise<void>;
}

export function StatusIndicator({
  error,
  hasKey,
  isChecking,
  label,
  onRetry,
}: StatusIndicatorProps) {
  const dotColor = error
    ? "bg-destructive"
    : hasKey === null
      ? "bg-gray-400 animate-pulse"
      : hasKey
        ? "bg-green-500"
        : "bg-destructive";

  // Show Retry whenever the check has produced an error OR whenever we are
  // still in the indeterminate `hasKey === null` state without an active
  // in-flight check (e.g. an IPC hang that bypassed our timeout). This
  // guarantees the user is never stuck on "Checking..." with no escape.
  const showRetry = !!onRetry && (!!error || (hasKey === null && !isChecking));

  return (
    <div className="flex items-center gap-3 p-3 rounded-md bg-muted/50 border border-border/50">
      <div className={`size-2.5 rounded-full shadow-sm ${dotColor}`} />
      <span
        role="status"
        aria-live="polite"
        aria-atomic="true"
        className="text-sm font-medium flex items-center gap-2 text-foreground"
      >
        {error ? (
          <>
            <AlertCircle className="h-4 w-4 text-destructive" />
            {`Failed to check ${label} Key status`}
          </>
        ) : hasKey === null ? (
          "Checking..."
        ) : hasKey ? (
          `${label} Key is active`
        ) : (
          `No ${label} Key found`
        )}
      </span>
      {showRetry && (
        <Button size="sm" variant="outline" className="ml-auto" onClick={() => onRetry?.()}>
          Retry
        </Button>
      )}
    </div>
  );
}
