import { Toaster } from "sonner";
import { LibraryView } from "@/features/library/LibraryView";

function App() {
  return (
    <div className="h-screen w-screen bg-background font-sans antialiased p-6">
      <LibraryView />
      <Toaster />
    </div>
  );
}

export default App;
