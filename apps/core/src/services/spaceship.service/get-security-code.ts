import { SECURITY_CODE_HEADER } from './constants';

const UUID_V4_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

export function getSecurityCode(headers: Record<string, unknown>) {
  const rawSecurityCode = headers[SECURITY_CODE_HEADER];
  const securityCode = Array.isArray(rawSecurityCode)
    ? rawSecurityCode[0]
    : rawSecurityCode;
  return securityCode && UUID_V4_PATTERN.test(securityCode)
    ? securityCode
    : undefined;
}

