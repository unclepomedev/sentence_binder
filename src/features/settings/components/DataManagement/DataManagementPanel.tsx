import { Database } from "lucide-react";
import { useBackup } from "../../hooks/useBackup";
import { ExportCard } from "./ExportCard";
import { ImportCard } from "./ImportCard";

export function DataManagementPanel() {
  const { exportData, importData, isExporting, isImporting } = useBackup();

  return (
    <section className="flex flex-col gap-4">
      <div className="px-1">
        <h2 className="text-lg font-semibold flex items-center gap-2 text-foreground">
          <Database className="w-5 h-5" />
          Data & Backups
        </h2>
        <p className="text-sm text-muted-foreground mt-1">
          Manage your sentence library. Export your data for safekeeping or import an existing
          backup.
        </p>
      </div>

      <div className="flex flex-col gap-3">
        <ExportCard onExport={exportData} isExporting={isExporting} />
        <ImportCard onImport={importData} isImporting={isImporting} />
      </div>
    </section>
  );
}
