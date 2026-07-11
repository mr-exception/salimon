export function cloneDate(value: Date | undefined) {
  return value ? new Date(value) : undefined;
}

