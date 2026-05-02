import { useState } from "react";
import { Card, CardContent } from "@/components/ui/card";
import type { Sentence } from "@/types";
import { SentenceCardHeader } from "./SentenceCardHeader";
import { SentenceCardViewer } from "./SentenceCardViewer";
import { SentenceEditForm } from "./SentenceEditForm";

interface SentenceCardProps {
  item: Sentence;
  isPlaying: boolean;
  isLocked: boolean;
  searchQuery?: string;
  onTogglePlay: () => void;
  onSaveEdit: (id: string, newText: string, newContext: string | null) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onStopAudio: () => Promise<void>;
}

export function SentenceCard({
  item,
  isPlaying,
  isLocked,
  searchQuery = "",
  onTogglePlay,
  onSaveEdit,
  onDelete,
  onStopAudio,
}: SentenceCardProps) {
  const [isEditing, setIsEditing] = useState(false);

  return (
    <Card className="bg-card shadow-sm group">
      <SentenceCardHeader
        originalText={item.original_text}
        isPlaying={isPlaying}
        isLocked={isLocked}
        searchQuery={searchQuery}
        onTogglePlay={onTogglePlay}
      />

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
          <SentenceCardViewer
            id={item.id}
            translatedText={item.translated_text}
            sourceContext={item.source_context}
            isPlaying={isPlaying}
            isLocked={isLocked}
            searchQuery={searchQuery}
            onEdit={() => setIsEditing(true)}
            onDelete={onDelete}
            onStopAudio={onStopAudio}
          />
        )}
      </CardContent>
    </Card>
  );
}
