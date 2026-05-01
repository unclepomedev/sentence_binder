import { AlertCircle, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ImportCardProps {
  onImport: () => void;
  isImporting: boolean;
}

export function ImportCard({ onImport, isImporting }: ImportCardProps) {
  return (
    <div className="flex items-start justify-between p-4 border rounded-xl bg-card text-card-foreground shadow-sm">
      <div className="space-y-3">
        <div>
          <h3 className="font-medium leading-none mb-1">Import Backup</h3>
          <p className="text-sm text-muted-foreground">
            Restore sentences from a previous JSON backup.
          </p>
        </div>
        <div className="flex items-center gap-2 text-xs text-amber-600 bg-amber-500/10 px-3 py-2 rounded-lg border border-amber-500/20 w-fit">
          <AlertCircle className="w-4 h-4 flex-none" />
          <span>Imported sentences are added to your existing library.</span>
        </div>
      </div>
      <Button
        variant="outline"
        onClick={onImport}
        disabled={isImporting}
        className="flex-none ml-4 w-36"
      >
        <Upload className="w-4 h-4 mr-2" />
        {isImporting ? "Importing..." : "Import JSON"}
      </Button>
    </div>
  );
}
