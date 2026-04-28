import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
import { IpcEvents } from "@/types/ipc";

export function useCapture(onCapture: (text: string) => void) {
  const onCaptureRef = useRef(onCapture);

  useEffect(() => {
    onCaptureRef.current = onCapture;
  }, [onCapture]);

  useEffect(() => {
    let unlistenPromise: Promise<UnlistenFn>;

    const setupListener = () => {
      unlistenPromise = listen<string>(IpcEvents.CAPTURE_TRIGGERED, (event) => {
        const text = event.payload.trim();
        if (text) {
          onCaptureRef.current(text);
        }
      });
    };

    setupListener();

    return () => {
      if (unlistenPromise) {
        unlistenPromise.then((unlisten) => unlisten());
      }
    };
  }, []);
}
