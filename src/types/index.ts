/** sentences table */
export type Sentence = {
  id: string;
  original_text: string;
  translated_text: string;
  source_context: string | null;
  /** milliseconds */
  created_at: number;
};
