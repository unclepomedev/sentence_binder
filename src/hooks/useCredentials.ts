import { useCredentialMutations } from "./credentials/useCredentialMutations";
import { useCredentialQuery } from "./credentials/useCredentialQuery";

/**
 * Public credentials API. Composes the read-side query (`useCredentialQuery`)
 * and write-side mutations (`useCredentialMutations`).
 */
// TODO: temporarily passing openai as provider. change it at a good time
export function useCredentials(provider: string = "openai") {
  const { hasKey, isChecking, isStuck, error, refresh, setHasKey, setError } =
    useCredentialQuery(provider);

  const { isSaving, isDeleting, saveKey, deleteKey } = useCredentialMutations({
    provider,
    refresh,
    setHasKey,
    setError,
  });

  return {
    hasKey,
    isChecking,
    isStuck,
    error,
    isSaving,
    isDeleting,
    saveKey,
    deleteKey,
    refresh,
  };
}
