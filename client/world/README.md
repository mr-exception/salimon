# World

Reserved for portable world data, scene state, and coordinate handling. Phase 0
will use the compressed, static Solar System specified in Notion. Keep world
state independent of GPU types and native window APIs so later clients and
backend code can reuse its source. No world simulation is implemented yet.
