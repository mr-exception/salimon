# Communications

## Overview

Communications is a ship feature that lets the player exchange text messages
with known non-player characters (NPCs). It appears alongside the ship's other
features and opens a chat dialog containing:

- A contact list with unread-message indicators.
- The selected contact's message history.
- A text field and send action.
- Sending, waiting, retry, and offline states.

Contacts are discovered through the story. A new game begins with one known
contact: the **Chief of EASA**. EASA stands for **Earth Aeronautics and Space
Administration**.

## Initial Contact

The Chief of EASA sends the first message when a spaceship is registered. The
canonical message is:

> Pilot, this is the Chief of EASA. An unknown being contacted humanity and
> gave us an energy cube with one message: “Reach Absenat, where the world is
> going to start.” We named the cube the Core. It produces the oxygen and
> electricity your ship needs, and both resources can be harvested aboard.
> Your mission is to leave Earth and reach Absenat. We do not yet know who sent
> the Core or what awaits you there. Stay in contact.

After this introduction, the player can continue talking to the Chief.

## NPC Conversations

Every contact has a server-owned NPC profile containing:

- Public identity: ID, display name, and organization.
- Private role, background, goal in the world, personality, and speaking style.
- Public game canon the NPC must not contradict.
- Unknowns the NPC must not answer with false certainty.
- Boundaries that prevent the NPC from changing game state or leaving the
  fiction.
- Story facts the NPC may reveal only after their unlock conditions are met.

NPC contacts are not normal helpful chatbots. Each one lives inside Salimon and
answers as that character, using their own motives, knowledge, pressure, and
voice. Out-of-world information does not matter to them unless it can be handled
in character as confusion, rumor, or irrelevant noise.

The backend generates NPC replies with the OpenAI Responses API. The browser
must never call OpenAI directly. For every reply, the backend supplies the NPC
profile, current authorized lore, a conversation summary, and recent messages
as context.

Conversation state is isolated by spaceship and contact. The complete message
history remains in the game database as the source of truth. A rolling summary
and a bounded number of recent messages should be sent to the model to control
latency and token usage without losing important context.

Model output is untrusted. Before storing a reply, the backend must enforce
length limits, reject empty output, and prevent the model from changing game
state. Story unlocks and resource changes happen only through game code, never
because an NPC reply claims they happened.

## HTTP API

All communication routes require the spaceship security code in the
`x-spaceship-security-code` header.

### `GET /contacts/info`

Returns the contacts known to the spaceship.

```json
{
  "contacts": [
    {
      "id": "easa-chief",
      "name": "Chief of EASA",
      "organization": "Earth Aeronautics and Space Administration",
      "unreadCount": 1,
      "lastMessageAt": "2026-07-05T12:00:00.000Z"
    }
  ]
}
```

Private NPC prompts and locked lore must never be included in this response.

### `GET /contacts/messages`

Returns a page of messages for one known contact.

Query parameters:

- `contactId`: Required contact identifier.
- `after`: Optional opaque cursor. Only messages newer than this cursor are
  returned.
- `limit`: Optional page size with a server-enforced maximum.

```json
{
  "messages": [
    {
      "id": "message-id",
      "contactId": "easa-chief",
      "sender": "contact",
      "text": "Message text",
      "createdAt": "2026-07-05T12:00:00.000Z"
    }
  ],
  "cursor": "opaque-next-cursor"
}
```

Reading returned contact messages marks them as read. The cursor is opaque so
the storage implementation can change without changing the client contract.

### `POST /contacts/messages/send`

Stores a player's message and schedules an NPC reply.

```json
{
  "contactId": "easa-chief",
  "text": "What do we know about Absenat?",
  "clientMessageId": "client-generated-uuid"
}
```

`clientMessageId` is required for idempotency, preventing duplicate messages
when the client retries. The route validates that the contact is known, trims
the text, applies a length limit and rate limit, and returns `202 Accepted`:

```json
{
  "message": {
    "id": "stored-message-id",
    "contactId": "easa-chief",
    "sender": "player",
    "text": "What do we know about Absenat?",
    "status": "sent",
    "createdAt": "2026-07-05T12:00:00.000Z"
  }
}
```

Generating the reply asynchronously avoids holding the HTTP request open for a
model response. A worker stores the NPC reply in the same conversation after
generation completes.

## Lambda Responsibilities

The feature is implemented as a small group of Lambda handlers:

| Handler                  | Responsibility                                                     |
| ------------------------ | ------------------------------------------------------------------ |
| `contacts-info`          | Authenticate the spaceship and list its unlocked contacts.         |
| `contact-messages`       | Read paginated messages and update read state.                     |
| `send-contact-message`   | Validate and store player messages, then enqueue reply generation. |
| `generate-contact-reply` | Load NPC context and history, call OpenAI, and store the reply.    |

The send handler should publish a job to SQS. The reply worker must use a
dead-letter queue and safe retry policy. A unique constraint on spaceship,
contact, and `clientMessageId` provides durable idempotency.

When a spaceship is registered, backend code unlocks `easa-chief` and stores
the Chief's initial mission message. Initial content is data, not generated
dynamically, so every player receives the canonical mission briefing.

## Polling Strategy

The first version uses HTTP polling:

1. Fetch contacts when the communication dialog opens.
2. Fetch the selected contact's messages immediately.
3. Poll selected-contact messages every 5 seconds using the latest cursor.
4. Poll contact metadata every 30 seconds for unread counts.
5. Poll immediately after sending a message.
6. Pause polling while the document is hidden or the browser is offline.
7. On failure, use exponential backoff up to 60 seconds with jitter.
8. Resume with an immediate poll when the document becomes visible or the
   network reconnects.

Only one poll per resource may be in flight. Responses are merged by message
ID, allowing retries and overlapping pages without duplicate chat entries.
WebSocket or server-sent-event delivery can replace polling later without
changing the persisted conversation model.

## Configuration and Secrets

The Lambda runtime requires:

| Variable         | Purpose                                    | Secret |
| ---------------- | ------------------------------------------ | ------ |
| `MONGODB_URI`    | Game and conversation database connection. | Yes    |
| `OPENAI_API_KEY` | Server-side OpenAI authentication.         | Yes    |
| `OPENAI_MODEL`   | Model ID used for NPC replies.             | No     |

Placeholders are provided in `apps/lambda/.env.example`. Real values belong in
local environment files, GitHub Actions secrets, or AWS Secrets Manager. Never
place `OPENAI_API_KEY` in a `VITE_*` variable, browser bundle, source file, log,
or committed environment file.

Production should pin `OPENAI_MODEL` deliberately and change it through a
reviewed deployment rather than relying on an implicit default.

## Delivery Requirements

- A new spaceship has exactly one contact, the Chief of EASA.
- The Chief's canonical mission briefing exists before the client first polls.
- The player can open Communications, read history, send a message, close the
  dialog, and later return to the same history.
- Replies respect the NPC profile, authorized lore, and conversation context.
- One spaceship cannot read another spaceship's contacts or messages.
- Duplicate send retries do not create duplicate player messages or replies.
- The UI indicates queued messages, unread messages, generation failures, and
  offline state.
- OpenAI credentials remain exclusively on the backend.
