# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based 3D action game built with the Bevy game engine. The game features a player character who fights waves of enemies in a top-down perspective with abilities, projectiles, and a rune/modifier system.

## Development Commands

### Comments by User
- always use the release profile
- avoid code duplication whenever possible
- you're a professional game developer
- prefer imports over fully qualified names (use `SubCastInfo::new()` instead of `components::subcast::SubCastInfo::new()`)

### Build and Run
- `cargo run` - Build and run the game in development mode
- `cargo build --release` - Build optimized release version
- `cargo check` - Quick compile check without building executable

### Testing
- `cargo test` - Run unit tests (limited test coverage currently)
- `cargo check --version` - Verify cargo and rustc versions

### Release Packaging
- `create_zip.bat` - Windows batch script that builds release, copies assets, and creates a distribution zip

## Architecture Overview

### Core Systems
The game is built around Bevy's Entity Component System (ECS) with modular plugin architecture:

- **State Management**: `GameState` enum with `Playing` and `GameOver` states
- **Game Loop**: Systems run conditionally based on game state
- **Plugin System**: Each major feature is organized as a Bevy plugin

### Key Modules

#### Core Game Systems
- `player.rs` - Player entity, movement, camera, and input handling
- `enemy/` - Enemy AI, spawning, and behavior systems
- `ability/` - Generic ability system with components for projectiles, blasts, dashes
- `projectile.rs` - Projectile movement, collision, and cleanup
- `common.rs` - Shared components like Health, Faction, damage handling

#### Game Features
- `rune/` - Collectible runes that provide gameplay modifiers
- `modifiers/` - Stat modification system for character upgrades
- `loot.rs` - Item drops and collection
- `fx/` - Visual effects including blood particles
- `audio/` - Music and sound effect management

#### UI Systems
- `ui/hud/` - In-game HUD elements (score, health, wave counter)
- `ui/menu/` - Game over screen and menu systems
- `ui/damage_numbers.rs` - Floating damage text
- `localization.rs` - Multi-language support using Fluent

#### Technical Systems
- `init.rs` - Game initialization and setup
- `input.rs` - Input handling and cursor management
- `event.rs` - Custom game events (damage, death, spawn)
- `procedural/` - Procedural content generation
- `utils.rs` - Utility functions

### Key Design Patterns

#### Generic Ability System
The ability system uses Rust generics to create reusable ability components:
- `AttemptCastEvent<T>` and `CastEvent<T>` for ability casting
- `AbilitySlot<T>` for cooldown management
- Trait-based system where each ability implements `Ability` trait

#### Component-Based Architecture
Abilities are built from composable components:
- `LinearMovement` - Projectile movement
- `DamageOnCollision` - Damage dealing
- `Lifetime` - Automatic cleanup
- `SpawnAtCastPosition` - Positioning logic

#### Event-Driven Systems
Game logic uses Bevy events for decoupled communication:
- `DamageEvent` - Entity takes damage
- `DeathEvent` - Entity dies
- `SpawnEvent` - Request entity spawn

### Physics and Collision
- Uses Avian3D physics engine
- Custom collision detection for projectiles
- Faction-based collision filtering

### Asset Management
- Assets stored in `assets/` directory
- Localization files in `assets/locale/`
- Audio files (.wav) and 3D models (.glb)
- Font assets for UI rendering

## Development Notes

### Key Dependencies
- `bevy = "0.16.0"` - Core game engine with audio features
- `avian3d = "0.3.1"` - Physics simulation
- `fluent`, `fluent-templates`, `unic-langid` - Localization system
- `fastrand = "2.3.0"` - Random number generation
- `noise = "0.9.0"` - Procedural generation

### Game State Flow
1. `GameState::Playing` - Main gameplay loop
2. `GameState::GameOver` - Death screen with restart option
3. Game resets to `Playing` state on restart

### Performance Considerations
- Uses ECS query filtering to minimize system overhead
- Conditional system execution based on game state
- Efficient projectile cleanup to prevent memory leaks
- Blood particle system with automatic cleanup

### Localization
- Multi-language support via Fluent localization files
- Current languages: English (en-US), German (de-DE), Swiss German (ch-CH)
- Rune pickup notifications are localized

## Common Development Tasks

When adding new enemies: Follow the pattern in `enemy/` module with spawn systems, AI behavior, and health components.

When adding new abilities: Use the generic ability system with composable components from `ability/components/`.

When modifying UI: UI elements are organized by function in `ui/` subdirectories with separate systems for setup/update/cleanup.

When adding audio: Use the event-driven audio system in `audio/` module.