export const IpcEvents = {
  CAPTURE_TRIGGERED: "double-tap-cmd-c",
} as const;

export type IpcEvent = (typeof IpcEvents)[keyof typeof IpcEvents];
