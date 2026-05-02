/** sentences table */
export type Sentence = {
  id: string;
  original_text: string;
  translated_text: string;
  source_context: string | null;
  tags: string[];
  /** milliseconds */
  created_at: number;
};
