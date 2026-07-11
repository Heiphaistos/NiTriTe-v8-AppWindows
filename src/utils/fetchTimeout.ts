// fetch() n'a aucun timeout par défaut : un hôte qui accepte la connexion sans
// répondre bloque l'appel à vie. Ce wrapper avorte la requête après timeoutMs.
// Note : le timeout couvre la connexion + les en-têtes. Pour un gros corps
// streamé (téléchargement), garder le contrôleur armé pendant la lecture du body.
export async function fetchWithTimeout(
  url: string,
  options: RequestInit = {},
  timeoutMs = 15000,
): Promise<Response> {
  const controller = new AbortController();
  const id = setTimeout(() => controller.abort(), timeoutMs);
  try {
    return await fetch(url, { ...options, signal: controller.signal });
  } finally {
    clearTimeout(id);
  }
}
