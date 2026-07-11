export function parseInvocationTime(time: string | Date) {
  const invocationTime = new Date(time);
  if (Number.isNaN(invocationTime.getTime())) {
    throw new Error('Invocation time is invalid');
  }
  return invocationTime;
}

