# Overall Game
- multi user
- dungeon crawler
- 2D/2.5D top down/bird's eye view
- a bit like zelda 4 swords?
- simple, royalty-free assets
  - public domain or permissive open source
- basic enemies
- inventory
- continuous coordinates
  - per-pixel effects??
  - physics system (acceleration, drag, etc)
  - rolling/movement options
  - bullet hell??? à la enter the gungeon
  - lighting system?
  - hitboxes: AABB or per-model geometry
- procedurally generated levels
- UI
  - HUD with all players' info (health, mana, ammo, etc)


# Technical
- bevy
- wasm target or not? would need to re-attack WebRTC for a UDP-like connection
- per-system logging and performance statistics


# Network
- UDP
- client-server architecture
- authoritative server
- client extrapolation
- server rollback too?


# Roadmap
- [x] set up git, cargo, github actions
- [x] render shapes, singleplayer WASD movement
- [x] walls, collisions
- [ ] add simple weapon and animation
- [ ] enemy, hit-/hurt-boxes
- [ ] (BIG) split into basic server/client
- [ ] simple enemy movement AI
- [ ] saving, loading, syncing state
- [ ] health, resources
- [ ] items + inventory
- [ ] ranged weapons, projectiles
- [ ] optimize netcode, extrapolation, interpolation, etc.
- [ ] levels, transitions
- [ ] shops? UIs
- [ ] menus, settings
- [ ] uPnP network port forwarding
- [ ] server browser??
- [ ] gamepad support
