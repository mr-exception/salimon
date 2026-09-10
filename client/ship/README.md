# Ship

Reserved for ship state, cockpit control, direct-speed flight, and assisted
landing/takeoff. Keep these behaviors independent of GPU resources and platform
APIs, with explicit interfaces for world and character interaction. No ship
behavior is implemented yet; Task 7's renderer-neutral source and runtime asset
live under [`client/assets/ship`](../assets/ship/README.md).
