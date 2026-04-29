import { toast } from "sonner";
import { Toaster } from "@/components/ui/sonner";
import { LibraryView } from "@/features/library/LibraryView";
import { TestButton } from "@/features/library/TestButton";
import { SettingsTest } from "@/features/settings/SettingsTest";
import { useCapture } from "@/hooks/useCapture";

function App() {
  useCapture((capturedText) => {
    toast.success("Text captured!", {
      description: capturedText,
    });
  });

  return (
    <div className="h-screen w-screen bg-background font-sans antialiased p-6 flex flex-col gap-4">
      <div className="flex-1 min-h-0">
        <LibraryView />
      </div>

      {import.meta.env.DEV && (
        <div className="flex-none flex flex-col gap-4">
          <TestButton />
          <SettingsTest />
        </div>
      )}

      <Toaster />
    </div>
  );
}

export default App;
