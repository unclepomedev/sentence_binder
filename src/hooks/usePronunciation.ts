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
      try {
        await invoke(IpcCommands.STOP_AUDIO);
        // Only clear the playing state once the stop has succeeded, so the UI
        // stays locked while the backend is still tearing down playback.
        if (playingIdRef.current === id) {
          setPlaying(null);
        }
      } catch (err) {
        console.error(err);
        toast.error("Failed to stop audio");
      }
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

  // Pure action: invokes STOP_AUDIO and clears playing state on success.
  // On failure, simply rethrows; callers are responsible for any user-facing
  // messaging so we don't double-toast for a single stop failure.
  const stopAudio = async () => {
    if (playingIdRef.current === null) return;
    await invoke(IpcCommands.STOP_AUDIO);
    setPlaying(null);
  };

  return { playingId, toggleAudio, stopAudio };
}
