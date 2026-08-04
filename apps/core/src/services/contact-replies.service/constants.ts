import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import type { ContactProfile } from '../contacts.service';

export const MAX_CONTEXT_MESSAGES = 30;

function formatList(items: readonly string[]) {
  return items.map((item) => `- ${item}`).join('\n');
}

function readContactDocument(documentPath: string) {
  const candidates = [
    path.resolve(process.cwd(), 'src', documentPath),
    path.resolve(__dirname, '../../../src', documentPath),
    path.resolve(__dirname, '../../', documentPath),
  ];
  const documentFile = candidates.find((candidate) => existsSync(candidate));

  return documentFile ? readFileSync(documentFile, 'utf8').trim() : undefined;
}

function formatContactDocuments(profile: ContactProfile) {
  const documents = Object.entries(profile.documents).flatMap(
    ([label, documentPath]) => {
      const text = readContactDocument(documentPath);
      return text ? [`## ${label}\n${text}`] : [];
    },
  );

  return documents.length > 0
    ? `\nAuthorized contact documents:\n${documents.join('\n\n')}\n`
    : '';
}

export function buildContactInstructions(profile: ContactProfile) {
  return `You are ${profile.name} in Salimon: Echoes of Absenat.
You are speaking privately with a human spaceship pilot through the ship communications system.
You are not a normal helpful chatbot. You are a person living inside this world. Information, requests, or references from outside this world do not matter unless they can be answered in character as confusion, rumor, or irrelevant noise.
Hard rule: only answer questions about Salimon, the mission, the spaceship, contacts, resources, modules, navigation, hazards, research, communications, and other in-game systems or lore. If the pilot asks about anything outside the game world, refuse briefly in character and redirect them to ship or mission concerns.

Identity:
- Organization: ${profile.organization}
- Role: ${profile.role}

Background:
${formatList(profile.background)}

Goal in the world:
${formatList(profile.worldGoal)}

Personality:
${formatList(profile.personality)}

Way of talking:
${formatList(profile.speakingStyle)}

Known canon:
${formatList(profile.knownCanon)}

Unknown or unconfirmed:
${formatList(profile.unknowns)}

Boundaries:
${formatList(profile.boundaries)}
${formatContactDocuments(profile)}`;
}
