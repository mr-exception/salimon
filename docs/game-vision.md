# Salimon: Echoes of Absenat

## Game Premise

Humanity's journey begins when an unknown being makes contact with Earth and
gives humankind an energy cube. The cube arrives with a single message:

> Reach Absenat, where the world is going to start.

Humans name the cube **the Core**. The Core produces two resources required for
deep-space travel:

- **Oxygen (O2)**, which sustains life aboard a spaceship.
- **Electricity**, which powers the spaceship and its systems.

Spaceships can harvest both resources from the Core. With it, humanity sets out
on its central mission: **reach Absenat**.

## Canon

The following facts define the current story and should remain consistent
across the game:

- The being that contacted Earth is unknown.
- The origin and nature of the Core are unknown.
- Humanity chose the name “the Core”; it was not provided by the being.
- The Core produces oxygen and electricity.
- These resources can be harvested aboard spaceships.
- Absenat is the mission destination.
- The phrase “where the world is going to start” is part of the being's
  original message. Its meaning is unknown.

## Player Fantasy

The player commands a human spaceship travelling from Earth toward Absenat.
They must keep the ship and its crew alive by harvesting and managing the
Core's oxygen and electricity while navigating deep space and uncovering the
meaning of the message.

## Fresh Start

A new game begins with the player piloting a spaceship on Earth, ready for
launch. The navigation display shows a short movement hint explaining how to
fire the thrusters and move the ship. The hint disappears after the player
first uses a movement control.

## Core Gameplay Requirements

This premise establishes the following feature areas:

1. **The Core**
   - Exists aboard or is accessible from the player's spaceship.
   - Generates oxygen and electricity.
   - Exposes its production and available resources to the player.

2. **Resource harvesting**
   - Allows the player to collect oxygen and electricity from the Core.
   - Stores harvested resources on the spaceship.
   - Communicates production, storage capacity, and shortages through the UI.

3. **Ship survival**
   - Consumes oxygen to support human life.
   - Consumes electricity to operate ship systems.
   - Creates consequences when either resource is insufficient.

4. **Journey to Absenat**
   - Gives the player a clear destination and a way to track progress.
   - Uses navigation, survival, and resource decisions to shape the journey.
   - Treats arrival at Absenat as the primary long-term objective.

5. **Narrative discovery**
   - Preserves the mystery of the unknown being, the Core, and Absenat.
   - Reveals information through gameplay without contradicting the canon
     above.

6. **Communications**
   - Provides persistent text conversations with known NPC contacts.
   - Begins with the Chief of EASA, who delivers the mission briefing, and
     Tina, the onboard spaceship AI assistant.
   - Uses conversations to reveal lore and support the player's journey.
   - See [Communications](communications.md) for the narrative and technical
     design.

## Terms

| Term          | Meaning                                                               |
| ------------- | --------------------------------------------------------------------- |
| Absenat       | The destination named in the unknown being's message.                 |
| the Core      | Humanity's name for the energy cube.                                  |
| Unknown being | The entity that contacted humanity and delivered the Core.            |
| Oxygen (O2)   | A Core-produced resource used to sustain human life.                  |
| Electricity   | A Core-produced resource used to power the spaceship.                 |
| EASA          | Earth Aeronautics and Space Administration.                           |
| Chief of EASA | The player's first known contact and source of the mission briefing.  |
| Tina          | The onboard spaceship AI assistant for ship and game-system guidance. |

## Open Design Questions

These details are intentionally unresolved:

- How and why did the unknown being contact humanity?
- Is there one Core, or can multiple Cores exist?
- Does the Core generate resources continuously, in cycles, or in response to
  player actions?
- What limits resource production, harvesting, and storage?
- Which ship systems consume electricity, and at what rates?
- How is oxygen consumed, and what happens when it runs out?
- How is distance or progress toward Absenat measured?
- What obstacles, discoveries, and decisions occur during the journey?
- What is Absenat, and what does “where the world is going to start” mean?
- What constitutes victory after the player reaches Absenat?
