import { Pencil, Square, Volume2 } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import type { Sentence } from "@/types";
import { SentenceEditForm } from "./SentenceEditForm";

interface SentenceCardProps {
  item: Sentence;
  isPlaying: boolean;
  isLocked: boolean;
  onTogglePlay: () => void;
  onSaveEdit: (id: string, newText: string, newContext: string | null) => Promise<void>;
}

export function SentenceCard({
  item,
  isPlaying,
  isLocked,
  onTogglePlay,
  onSaveEdit,
}: SentenceCardProps) {
  const [isEditing, setIsEditing] = useState(false);

  return (
    <Card className="bg-card shadow-sm group">
      <CardHeader className="pb-2 flex flex-row items-start justify-between space-y-0">
        <p className="text-base font-medium leading-tight pr-4">{item.original_text}</p>
        <Button
          variant="ghost"
          size="icon"
          disabled={isLocked && !isPlaying}
          className="h-8 w-8 text-muted-foreground hover:text-foreground shrink-0 transition-colors"
          onClick={onTogglePlay}
        >
          {isPlaying ? (
            <Square className="h-4 w-4 fill-current text-foreground" />
          ) : (
            <Volume2 className="h-4 w-4" />
          )}
          <span className="sr-only">{isPlaying ? "Stop pronunciation" : "Play pronunciation"}</span>
        </Button>
      </CardHeader>

      <CardContent>
        {isEditing ? (
          <SentenceEditForm
            initialText={item.translated_text}
            initialContext={item.source_context}
            onSave={(newText, newContext) =>
              onSaveEdit(item.id, newText, newContext).then(() => setIsEditing(false))
            }
            onCancel={() => setIsEditing(false)}
          />
        ) : (
          <div className="relative">
            <p
              className={`text-sm ${!item.translated_text ? "text-destructive italic" : "text-muted-foreground"}`}
            >
              {item.translated_text || "[ Translation Failed ]"}
            </p>

            <div className="flex items-end justify-between mt-3">
              <div className="flex gap-1 opacity-0 focus-within:opacity-100 group-hover:opacity-100 transition-opacity">
                <Button
                  variant="outline"
                  size="icon"
                  className="h-7 w-7"
                  onClick={() => setIsEditing(true)}
                  title="Manual Edit"
                  aria-label="Edit sentence"
                >
                  <Pencil className="h-3.5 w-3.5 text-muted-foreground" />
                  <span className="sr-only">Edit sentence</span>
                </Button>
              </div>

              {item.source_context && (
                <p className="text-[10px] text-muted-foreground/60 text-right truncate max-w-[70%] ml-auto">
                  {item.source_context}
                </p>
              )}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
