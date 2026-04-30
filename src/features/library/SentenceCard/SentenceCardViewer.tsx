import { confirm } from "@tauri-apps/plugin-dialog";
import { Pencil, Trash2 } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";

interface SentenceCardViewerProps {
  id: string;
  translatedText: string;
  sourceContext: string | null;
  isPlaying: boolean;
  isLocked: boolean;
  onEdit: () => void;
  onDelete: (id: string) => Promise<void>;
  onStopAudio: () => Promise<void>;
}

export function SentenceCardViewer({
  id,
  translatedText,
  sourceContext,
  isPlaying,
  isLocked,
  onEdit,
  onDelete,
  onStopAudio,
}: SentenceCardViewerProps) {
  // Disable delete while another card holds the audio lock to avoid orphaning
  // the only stop control. If this card is the one currently playing, stop
  // audio first so playback doesn't outlive the deleted record.
  const [isDeleting, setIsDeleting] = useState(false);
  const deleteDisabled = (isLocked && !isPlaying) || isDeleting;

  const handleDelete = async () => {
    if (isDeleting) return;
    if (deleteDisabled) return;

    const isConfirmed = await confirm("Are you sure you want to delete this sentence?", {
      title: "Delete Sentence",
      kind: "warning",
    });

    if (!isConfirmed) return;

    setIsDeleting(true);
    try {
      if (isPlaying) {
        try {
          await onStopAudio();
        } catch {
          // If stopping fails, abort the delete so the user isn't left with a
          // locked UI and a deleted record.
          return;
        }
      }

      try {
        await onDelete(id);
      } catch (err) {
        // Error is handled/toasted upstream in useDeleteSentence; log here
        // to avoid an uncaught async error from the click handler.
        console.error(err);
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
        {translatedText || "[ Translation Failed ]"}
      </p>

      <div className="flex items-end justify-between mt-3">
        <div className="flex gap-1 opacity-0 focus-within:opacity-100 group-hover:opacity-100 transition-opacity">
          <Button
            variant="outline"
            size="icon"
            className="h-7 w-7 text-muted-foreground hover:text-foreground transition-colors"
            onClick={onEdit}
            title="Manual Edit"
            aria-label="Edit sentence"
          >
            <Pencil className="h-3.5 w-3.5" />
            <span className="sr-only">Edit sentence</span>
          </Button>

          <Button
            variant="outline"
            size="icon"
            className="h-7 w-7 text-muted-foreground hover:text-foreground transition-colors"
            onClick={handleDelete}
            disabled={deleteDisabled}
            title={
              deleteDisabled ? "Stop audio before deleting" : "Delete Sentence"
            }
            aria-label="Delete sentence"
          >
            <Trash2 className="h-3.5 w-3.5" />
            <span className="sr-only">Delete sentence</span>
          </Button>
        </div>

        {sourceContext && (
          <p className="text-[10px] text-muted-foreground/60 text-right truncate max-w-[70%] ml-auto">
            {sourceContext}
          </p>
        )}
      </div>
    </div>
  );
}
