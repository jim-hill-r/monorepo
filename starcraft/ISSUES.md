# Priority Issues

TODO (agent-generated): Research Bevy ECS map cleanup lifecycle and document current behavior of process_loaded_maps system, including when tiles/layers are spawned vs despawned
TODO (agent-generated): Design RemoveMap component API - determine if it should be a marker component or hold cleanup metadata
TODO (agent-generated): Implement separate cleanup system that processes RemoveMap components to despawn map entities
TODO (agent-generated): Refactor process_loaded_maps to use RemoveMap component instead of manual despawn logic
TODO (agent-generated): Add unit/integration tests for map loading, reloading, and removal scenarios
TODO (agent-generated): Document the map entity lifecycle and cleanup process in code comments

# Backlog

TODO (agent-ignore): Create isometric map (use bevy_ecs_tilemap)
