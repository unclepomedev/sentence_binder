import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { toast } from "sonner";
import { IpcCommands } from "@/types/ipc";

export function useBackup() {
  const queryClient = useQueryClient();
  const [isExporting, setIsExporting] = useState(false);
  const [isImporting, setIsImporting] = useState(false);

  const exportData = async () => {
    setIsExporting(true);
    try {
      await invoke(IpcCommands.EXPORT_SENTENCES_JSON);
      // backend command returns `Ok(())` even if the user cancels the dialog,
      // so don't necessarily need a success toast here.
    } catch (err) {
      console.error(err);
      toast.error("Failed to export sentences");
    } finally {
      setIsExporting(false);
    }
  };

  const importData = async () => {
    setIsImporting(true);
    try {
      const count = await invoke<number>(IpcCommands.IMPORT_SENTENCES_JSON);
      if (count > 0) {
        toast.success(`Successfully imported ${count} sentences`);
        void queryClient.invalidateQueries({ queryKey: ["sentences"] });
      }
    } catch (err) {
      console.error(err);
      toast.error("Failed to import sentences");
    } finally {
      setIsImporting(false);
    }
  };

  return { exportData, importData, isExporting, isImporting };
}
