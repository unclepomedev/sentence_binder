import { invoke } from "@tauri-apps/api/core";
import { useRef, useState } from "react";
import { toast } from "sonner";
import { IpcCommands } from "@/types/ipc";

export function usePronunciation() {
  const [playingId, setPlayingId] = useState<string | null>(null);
  const playingIdRef = useRef<string | null>(null);

  const setPlaying = (next: string | null) => {
    playingIdRef.current = next;
    setPlayingId(next);
  };

  const toggleAudio = async (id: string, text: string) => {
    if (playingIdRef.current === id) {
      setPlaying(null);
      await invoke(IpcCommands.STOP_AUDIO);
      return;
    }

    if (playingIdRef.current) return;

    setPlaying(id);

    try {
      await invoke(IpcCommands.PLAY_PRONUNCIATION, { text });
    } catch (err) {
      if (playingIdRef.current === id) {
        console.error(err);
        toast.error("Failed to play audio");
      }
    } finally {
      if (playingIdRef.current === id) {
        setPlaying(null);
      }
    }
  };

  return { playingId, toggleAudio };
}
