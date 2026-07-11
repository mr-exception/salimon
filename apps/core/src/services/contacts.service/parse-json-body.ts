export function parseJsonBody(body: unknown) {
  if (!body || typeof body !== 'object' || Array.isArray(body)) {
    throw new Error('Request body must be an object');
  }
  return body as Record<string, unknown>;
}

