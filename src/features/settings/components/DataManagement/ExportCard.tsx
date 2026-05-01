import { Download } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ExportCardProps {
  onExport: () => void;
  isExporting: boolean;
}

export function ExportCard({ onExport, isExporting }: ExportCardProps) {
  return (
    <div className="flex items-start justify-between p-4 border rounded-xl bg-card text-card-foreground shadow-sm">
      <div className="space-y-1">
        <h3 className="font-medium leading-none">Export Library</h3>
        <p className="text-sm text-muted-foreground">
          Save your entire sentence library as a JSON file.
        </p>
      </div>
      <Button
        variant="outline"
        onClick={onExport}
        disabled={isExporting}
        className="flex-none ml-4 w-36"
      >
        <Download className="w-4 h-4 mr-2" />
        {isExporting ? "Exporting..." : "Export JSON"}
      </Button>
    </div>
  );
}
