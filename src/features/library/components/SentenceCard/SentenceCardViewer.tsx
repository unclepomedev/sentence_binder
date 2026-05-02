import { confirm } from "@tauri-apps/plugin-dialog";
import { Pencil, Trash2 } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { HighlightText } from "@/components/ui/highlight-text";

interface SentenceCardViewerProps {
  id: string;
  translatedText: string;
  sourceContext: string | null;
  isPlaying: boolean;
  isLocked: boolean;
  searchQuery?: string;
  onEdit: () => void;
  onDelete: (id: string) => Promise<void>;
  onStopAudio: () => Promise<void>;
  tags: string[];
  onTagClick: (tag: string) => void;
}

export function SentenceCardViewer({
  id,
  translatedText,
  sourceContext,
  isPlaying,
  isLocked,
  searchQuery = "",
  onEdit,
  onDelete,
  onStopAudio,
  tags = [],
  onTagClick,
}: SentenceCardViewerProps) {
  // Disable delete while another card holds the audio lock to avoid orphaning
  // the only stop control. If this card is the one currently playing, stop
  // audio first so playback doesn't outlive the deleted record.
  const [isDeleting, setIsDeleting] = useState(false);
  const deleteDisabled = (isLocked && !isPlaying) || isDeleting;

  const handleDelete = async () => {
    if (isDeleting || deleteDisabled) return;

    // Lock immediately to prevent parallel delete flows from concurrent clicks
    // racing through the (async) confirm dialog before isDeleting is set.
    setIsDeleting(true);
    try {
      let isConfirmed = false;
      try {
        isConfirmed = await confirm("Are you sure you want to delete this sentence?", {
          title: "Delete Sentence",
          kind: "warning",
        });
      } catch (err) {
        console.error(err);
        toast.error("Failed to open delete confirmation");
        return;
      }

      if (!isConfirmed) return;

      if (isPlaying) {
        try {
          await onStopAudio();
        } catch (err) {
          console.error(err);
          toast.error("Failed to stop audio playback. Deletion canceled.");
          return;
        }
      }

      try {
        await onDelete(id);
      } catch {
        // Handled/toasted upstream in useDeleteSentence
      }
    } finally {
      setIsDeleting(false);
    }
  };

  return (
    <div className="relative">
      <p
        className={`text-sm ${!translatedText ? "text-destructive italic" : "text-muted-foreground"}`}
      >
        {translatedText ? (
          <HighlightText text={translatedText} query={searchQuery} />
        ) : (
          "[ Translation Failed ]"
        )}
      </p>

      {tags.length > 0 && (
        <div className="flex flex-wrap gap-1.5 mt-3">
          {tags.map((tag) => (
            <Badge
              key={tag}
              variant="secondary"
              className="px-2 py-0.5 text-[10px] cursor-pointer hover:bg-secondary/80 transition-colors"
              onClick={() => onTagClick(tag)}
            >
              <HighlightText text={tag} query={searchQuery} />
            </Badge>
          ))}
        </div>
      )}

      <div className="flex items-end justify-between mt-3">
        <div className="flex gap-1 opacity-0 focus-within:opacity-100 group-hover:opacity-100 transition-opacity">
          <Button
            variant="outline"
            size="icon-sm"
            className="text-muted-foreground hover:text-foreground transition-colors"
            onClick={onEdit}
            title="Manual Edit"
            aria-label="Edit sentence"
          >
            <Pencil />
          </Button>

          <Button
            variant="outline"
            size="icon-sm"
            className="text-muted-foreground hover:text-foreground transition-colors"
            onClick={handleDelete}
            disabled={deleteDisabled}
            title={deleteDisabled ? "Stop audio before deleting" : "Delete Sentence"}
            aria-label="Delete sentence"
          >
            <Trash2 />
          </Button>
        </div>

        {sourceContext && (
          <p className="text-[10px] text-muted-foreground/60 text-right truncate max-w-[70%] ml-auto">
            <HighlightText text={sourceContext} query={searchQuery} />
          </p>
        )}
      </div>
    </div>
  );
}
