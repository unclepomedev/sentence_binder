import type * as React from "react";

interface HighlightTextProps extends React.HTMLAttributes<HTMLSpanElement> {
  text?: string | null;
  /**
   * The search query to highlight. Callers must pass a pre-normalized (trimmed) value;
   * pass an empty string (or omit) to disable highlighting.
   */
  query?: string;
}

export function HighlightText({ text, query, className, ...props }: HighlightTextProps) {
  if (!text) return null;

  const normalizedQuery = query ?? "";
  if (!normalizedQuery) {
    return (
      <span className={className} {...props}>
        {text}
      </span>
    );
  }

  const escapedQuery = normalizedQuery.replace(/[-[\]{}()*+?.,\\^$|#\s]/g, "\\$&");
  const parts = text.split(new RegExp(`(${escapedQuery})`, "gi"));

  return (
    <span className={className} {...props}>
      {parts.map((part, index) => {
        const uniqueKey = `${part}-${index}`;

        return part.toLowerCase() === normalizedQuery.toLowerCase() ? (
          <mark
            key={uniqueKey}
            className="bg-yellow-400/40 text-foreground rounded-xs px-0.5 font-medium transition-colors"
          >
            {part}
          </mark>
        ) : (
          <span key={uniqueKey}>{part}</span>
        );
      })}
    </span>
  );
}
