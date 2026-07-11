export function queryString(value: unknown) {
  return typeof value === 'string' ? value : undefined;
}
