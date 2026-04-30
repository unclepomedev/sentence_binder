import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
import { type CapturePayload, IpcEvents } from "@/types/ipc";

export function useCapture(onCapture: (payload: CapturePayload) => void) {
  const onCaptureRef = useRef(onCapture);

  useEffect(() => {
    onCaptureRef.current = onCapture;
  }, [onCapture]);

  useEffect(() => {
    let unlistenPromise: Promise<UnlistenFn>;

    const setupListener = () => {
      unlistenPromise = listen<CapturePayload>(IpcEvents.CAPTURE_TRIGGERED, (event) => {
        const payload = event.payload;
        if (payload.text?.trim()) {
          onCaptureRef.current({
            text: payload.text.trim(),
            context: payload.context,
          });
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
