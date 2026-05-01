import { Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardAction, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

interface ExportCardProps {
  onExport: () => void;
  isExporting: boolean;
}

export function ExportCard({ onExport, isExporting }: ExportCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Export Library</CardTitle>
        <CardDescription>Save your entire sentence library as a JSON file.</CardDescription>
        <CardAction>
          <Button variant="outline" onClick={onExport} disabled={isExporting} className="w-36">
            <Download />
            {isExporting ? "Exporting..." : "Export JSON"}
          </Button>
        </CardAction>
      </CardHeader>
    </Card>
  );
}
