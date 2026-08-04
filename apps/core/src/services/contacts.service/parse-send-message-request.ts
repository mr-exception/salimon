import type { ContactShipContext } from './constants';

const MAX_MESSAGE_LENGTH = 1_000;
const MAX_SHIP_CONTEXT_LENGTH = 12_000;
const UUID_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

export function parseSendMessageRequest(body: Record<string, unknown>) {
  const contactId =
    typeof body.contactId === 'string' ? body.contactId.trim() : '';
  const text = typeof body.text === 'string' ? body.text.trim() : '';
  const clientMessageId =
    typeof body.clientMessageId === 'string' ? body.clientMessageId.trim() : '';

  if (!contactId) throw new Error('contactId is required');
  if (!text) throw new Error('text is required');
  if (text.length > MAX_MESSAGE_LENGTH) {
    throw new Error(`text must be at most ${MAX_MESSAGE_LENGTH} characters`);
  }
  if (!UUID_PATTERN.test(clientMessageId)) {
    throw new Error('clientMessageId must be a UUID');
  }

  return {
    contactId,
    text,
    clientMessageId,
    shipContext: parseShipContext(body.shipContext),
  };
}

function parseShipContext(value: unknown): ContactShipContext | undefined {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return undefined;
  }

  if (JSON.stringify(value).length > MAX_SHIP_CONTEXT_LENGTH) {
    throw new Error(
      `shipContext must be at most ${MAX_SHIP_CONTEXT_LENGTH} characters`,
    );
  }

  return value as ContactShipContext;
}
