# 🟠 Orange Stone

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Phase](https://img.shields.io/badge/phase-3-blue)]()

*中文版: [README_zh.md](README_zh.md)*

A high-performance Hearthstone simulator written in Rust, purpose-built for Reinforcement Learning training.

## Why Orange Stone?

Existing Hearthstone simulators ([RosettaStone](https://github.com/utilForever/RosettaStone), [SabberStone](https://github.com/HearthSim/SabberStone)) focus on game replay and AI-vs-AI matches. Orange Stone has a different goal — it's designed for **massively parallel simulation**, targeting:

- Simulating tens of thousands of games per step to collect experience for RL training
- Deterministic state replay for experience replay and debugging
- Seamless integration with the Python ML ecosystem (PyTorch, JAX, RLlib)
- Batched encoding of game states into tensors for neural network consumption

Rust's ownership system is a natural fit: zero-cost abstractions for batch simulation, predictable latency without GC pauses, and compile-time concurrency safety.

## Current Progress

- ✅ **Phase 1**: Core framework — ECS, GameState (CoW), basic Action/Event loop
- ✅ **Phase 2**: Basic rules — mana crystals, turn flow, Battlecry, Deathrattle, Taunt, Divine Shield, Charge, Windfury
- 🔄 **Phase 3**: Full rules — Auras, Secrets, Weapons, Hero Powers, Choose One/Combo, 316 unique Classic cards
- ⬜ **Phase 4**: RL interface — Gym API, batched simulation, tensor observations, reward functions
- ⬜ **Phase 5**: Performance optimization — SIMD, zero-copy deserialization, GPU batched inference

## Design Highlights

### ECS Architecture

Instead of traditional inheritance trees (`Minion → Weapon → Spell`), Orange Stone uses Entity Component System, decomposing card effects into composable Components:

```rust
// A "Battlecry: Draw a card" minion
entity
    .with(Health(5))
    .with(Attack(3))
    .with(Cost(4))
    .with(Battlecry(DrawCard(1)))
    .with(Zone(ZoneType::Hand));
```

This enables flexible effect composition, cache-friendly data layout (SoA), and SIMD-friendly batch operations.

### Immutable State + Copy-on-Write

Every state change produces a new `GameState`, sharing unchanged parts via CoW:

```
state_0 ──→ state_1 ──→ state_2
  │                       │
  └───→ branch_1 ──→ branch_2   (MCTS branching, near-zero overhead)
```

Critical for RL: MCTS search, experience replay, and distributed training all depend on efficient state cloning and rollback.

### Event-Driven Rule Engine

```
[Action] → [Event] → [Trigger match] → [New Event] → ...
```

Actions (play card, attack, end turn) generate Events. Triggers (Battlecry, Deathrattle, Aura, Secret) respond to Events by generating new ones, resolved in a priority queue until the queue is empty.

### Performance

| Metric | Current | Target |
|--------|---------|--------|
| Single-core speed | ~7,000 matches/s | > 50,000 steps/s |
| Card coverage | 316 Classic cards | Full Standard set |
| Memory usage | < 1 MB / instance | < 1 MB / game instance |
| Integration tests | 131 | Continuously growing |

## Quick Start

### Prerequisites

- Rust 1.85+

### Build

```bash
git clone https://github.com/yorange2/orange-stone.git
cd orange-stone
cargo build --release
```

### Run Tests

```bash
# All tests
cargo test

# 1000-round bot battle (massive card coverage test)
cargo test run_1000 --release -- --nocapture

# Visual bot battle replay
cargo test bot_game -- --nocapture
```

### Python Bindings (Planned)

```python
import orange_stone as os

# Create batched environment
env = os.BatchedGameEnv(num_envs=1024)

# Reset all environments
states = env.reset()

# RL training loop
for step in range(max_steps):
    actions = agent.compute_actions(states)
    next_states, rewards, terminals, infos = env.step(actions)
    ...
```

## Architecture

```
src/
├── core/           # ECS core: Entity, Component, World, GameState (CoW)
├── engine/         # Rule engine: validation, event queue, effect resolution, auras, secrets
├── cards/          # Card definitions: CardDef, vanilla! macro, 316 Classic cards
├── sim/            # Simulation layer: GameBuilder, GreedyBot, SmartBot, RNG
└── lib.rs
tests/
├── gameplay.rs     # 36 rule engine integration tests
├── bot_game.rs     # 2 visual bot battle tests
└── battle_1000.rs  # Large-scale card coverage test (1000 rounds)
```

## Contributing

Contributions welcome! Please read [CLAUDE.md](CLAUDE.md) for project architecture and coding conventions.

- Use Chinese for commit messages
- All public APIs must have doc comments
- Core logic requires unit tests
- New card effects should include property tests

## Acknowledgments

- [RosettaStone](https://github.com/utilForever/RosettaStone) — C++ Hearthstone simulator; key reference for rule engine design
- [SabberStone](https://github.com/HearthSim/SabberStone) — C# Hearthstone simulator; key reference for card implementations
- [HearthSim](https://hearthsim.info/) — Hearthstone simulation community

## License

MIT License
