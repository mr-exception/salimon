export const EASA_CHIEF_ID = 'easa-chief';
export const INITIAL_CHIEF_MESSAGE =
  'Pilot, this is the Chief of EASA. An unknown being contacted humanity and gave us an energy cube with one message: “Reach Absenat, where the world is going to start.” We named the cube the Core. It produces the oxygen and electricity your ship needs, and both resources can be harvested aboard. Your mission is to leave Earth and reach Absenat. We do not yet know who sent the Core or what awaits you there. Stay in contact.';
export const CONTACTS = {
  [EASA_CHIEF_ID]: {
    id: EASA_CHIEF_ID,
    name: 'Chief of EASA',
    organization: 'Earth Aeronautics and Space Administration',
  },
} as const;

