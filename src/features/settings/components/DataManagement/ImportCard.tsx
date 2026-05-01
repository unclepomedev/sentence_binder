import { AlertCircle, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

interface ImportCardProps {
  onImport: () => void;
  isImporting: boolean;
}

export function ImportCard({ onImport, isImporting }: ImportCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Import Backup</CardTitle>
        <CardDescription>Restore sentences from a previous JSON backup.</CardDescription>
        <CardAction>
          <Button variant="outline" onClick={onImport} disabled={isImporting} className="w-36">
            <Upload />
            {isImporting ? "Importing..." : "Import JSON"}
          </Button>
        </CardAction>
      </CardHeader>
      <CardContent>
        <div className="flex items-center gap-2 text-xs text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-500/10 px-3 py-2 rounded-lg border border-amber-200 dark:border-amber-500/20 w-fit">
          <AlertCircle className="w-4 h-4 flex-none" />
          <span>Imported sentences are added to your existing library.</span>
        </div>
      </CardContent>
    </Card>
  );
}
