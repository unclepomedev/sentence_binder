import { Database } from "lucide-react";
import { useBackup } from "../../hooks/useBackup";
import { ExportCard } from "./ExportCard";
import { ImportCard } from "./ImportCard";

export function DataManagementPanel() {
  const { exportData, importData, isExporting, isImporting } = useBackup();

  return (
    <div className="space-y-6 animate-in fade-in duration-300">
      <div>
        <h2 className="text-xl font-semibold tracking-tight flex items-center gap-2 mb-1">
          <Database className="w-5 h-5" />
          Data & Backups
        </h2>
        <p className="text-sm text-muted-foreground">
          Manage your sentence library. Export your data for safekeeping or import an existing
          backup.
        </p>
      </div>

      <div className="space-y-4">
        <ExportCard onExport={exportData} isExporting={isExporting} />
        <ImportCard onImport={importData} isImporting={isImporting} />
      </div>
    </div>
  );
}
