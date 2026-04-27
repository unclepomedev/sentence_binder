/** sentences table */
export type Sentence = {
  id: string;
  original_text: string;
  translated_text: string;
  source_context: string | null;
  audio_file_name: string | null;
  /** milliseconds */
  created_at: number;
};
