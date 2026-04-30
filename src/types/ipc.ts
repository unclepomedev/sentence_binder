export const IpcEvents = {
  // Must match `EVENT_CAPTURE_TRIGGERED` in `src-tauri/src/constants.rs`.
  CAPTURE_TRIGGERED: "double-tap-cmd-c",
} as const;

export type IpcEvent = (typeof IpcEvents)[keyof typeof IpcEvents];

// The payload for the CAPTURE_TRIGGERED event
export interface CapturePayload {
  text: string;
  context: string | null;
}

export const IpcCommands = {
  SAVE_API_KEY: "save_api_key",
  HAS_API_KEY: "has_api_key",
  DELETE_API_KEY: "delete_api_key",
  SAVE_SENTENCE: "save_sentence",
  GET_SENTENCES: "get_sentences",
  PLAY_PRONUNCIATION: "play_pronunciation",
  STOP_AUDIO: "stop_audio",
} as const;

export type IpcCommand = (typeof IpcCommands)[keyof typeof IpcCommands];
