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

  const cleanQuery = normalizedQuery.replace(/tag:/gi, "");
  const terms = cleanQuery.split(/\s+/).filter(Boolean);

  if (terms.length === 0) {
    return (
      <span className={className} {...props}>
        {text}
      </span>
    );
  }

  const escapedTerms = terms.map((t) => t.replace(/[-[\]{}()*+?.,\\^$|#]/g, "\\$&"));
  const splitRegex = new RegExp(`(${escapedTerms.join("|")})`, "gi");
  const matchRegex = new RegExp(`^(?:${escapedTerms.join("|")})$`, "i");
  const parts = text.split(splitRegex);

  return (
    <span className={className} {...props}>
      {parts.map((part, index) => {
        const uniqueKey = `${part}-${index}`;

        return matchRegex.test(part) ? (
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
