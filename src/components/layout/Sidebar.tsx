import { BookOpen, Settings } from "lucide-react";
import { Button } from "@/components/ui/button";

export type ViewState = "library" | "settings";

interface SidebarProps {
  currentView: ViewState;
  onViewChange: (view: ViewState) => void;
}

export function Sidebar({ currentView, onViewChange }: SidebarProps) {
  return (
    <nav className="w-16 flex-none border-r bg-muted/10 flex flex-col items-center py-6 gap-4">
      <Button
        variant={currentView === "library" ? "secondary" : "ghost"}
        size="icon"
        className="h-10 w-10 rounded-xl"
        onClick={() => onViewChange("library")}
        title="Library"
        aria-pressed={currentView === "library"}
      >
        <BookOpen className="h-5 w-5" />
        <span className="sr-only">Library</span>
      </Button>

      <Button
        variant={currentView === "settings" ? "secondary" : "ghost"}
        size="icon"
        className="h-10 w-10 rounded-xl"
        onClick={() => onViewChange("settings")}
        title="Settings"
        aria-pressed={currentView === "settings"}
      >
        <Settings className="h-5 w-5" />
        <span className="sr-only">Settings</span>
      </Button>
    </nav>
  );
}
