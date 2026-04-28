export const IpcEvents = {
  CAPTURE_TRIGGERED: "double-tap-cmd-c",
} as const;

export type IpcEvent = (typeof IpcEvents)[keyof typeof IpcEvents];

export const IpcCommands = {
  SAVE_API_KEY: "save_api_key",
  GET_API_KEY: "get_api_key",
  DELETE_API_KEY: "delete_api_key",
  SAVE_SENTENCE: "save_sentence",
  GET_SENTENCES: "get_sentences",
} as const;

export type IpcCommand = (typeof IpcCommands)[keyof typeof IpcCommands];
