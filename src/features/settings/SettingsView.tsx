import { DataManagementPanel } from "./components/DataManagement";
import { WebLLMSettings } from "./components/WebLLMSettings";

export function SettingsView() {
  return (
    <div className="flex flex-col h-full gap-4 overflow-hidden">
      <header className="flex-none">
        <h1 className="text-2xl font-bold tracking-tight">Preferences</h1>
      </header>

      <main className="flex-1 flex flex-col min-h-0 overflow-y-auto pb-6">
        <div className="flex flex-col gap-6">
          <WebLLMSettings />
          <DataManagementPanel />
          {/* TODO: Future sections (e.g., <AudioSettings />, <AppearanceSettings />) will go here */}
        </div>
      </main>
    </div>
  );
}
