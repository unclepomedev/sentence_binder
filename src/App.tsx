import { toast } from "sonner";
import { Toaster } from "@/components/ui/sonner";
import { LibraryView } from "@/features/library/LibraryView";
import { useCapture } from "@/hooks/useCapture";

function App() {
  useCapture((capturedText) => {
    toast.success("Text captured!", {
      description: capturedText,
    });
  });

  return (
    <div className="h-screen w-screen bg-background font-sans antialiased p-6">
      <LibraryView />
      <Toaster />
    </div>
  );
}

export default App;
