export const SENTENCES_QUERY_KEY_ROOT = "sentences" as const;

export function sentencesQueryKey(searchQuery: string = "") {
  return [SENTENCES_QUERY_KEY_ROOT, searchQuery] as const;
}
