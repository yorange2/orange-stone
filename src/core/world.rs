//! World — the ECS container that manages all entities and components.
//!
//! World is the single entry point for entity lifecycle:
//! - `spawn()` creates an entity (allocates a slot)
//! - `despawn()` destroys an entity (frees the slot, clears all components and zone references)
//! - `move_to_zone()` atomically moves an entity from one zone to another
//! - components are accessed through generated accessor methods
//!
//! All entity access goes through a generation check to prevent dangling references.
use serde::{Deserialize, Serialize};

use crate::core::component::{
    Armor, Attack, AttackEqualsHealth, AttacksUsed, Aura, Battlecry, CantAttack, CardId, CardType,
    Charge, ChooseOneEffect, ComboEffect, Cost, CostModifier, CostModifierKind, Damage,
    Deathrattle, DivineShield, Durability, Elusive, Enchantment, Enrage, Freeze, Health,
    HeroPowerDef, HeroPowerUsed, Immune, Lifesteal, Overload, Poison, Race, Reborn, Rush, Secret,
    SpellDamage, Stealth, SummonedThisTurn, Taunt, Trigger, Windfury,
};
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::sparse_set::SparseSet;
use crate::core::zone::{Zone, ZoneError, Zones};

/// Zone move error — the possible failure modes of move_to_zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// The entity has been destroyed
    EntityGone,
    /// The PlayerId component is missing
    MissingPlayer,
    /// The current Zone component is missing
    MissingZone,
}

impl From<ZoneError> for MoveError {
    fn from(e: ZoneError) -> Self {
        match e {
            ZoneError::EntityGone => Self::EntityGone,
            ZoneError::MissingPlayer => Self::MissingPlayer,
            ZoneError::MissingZone => Self::MissingZone,
        }
    }
}

/// Macro that generates component accessor methods.
///
/// Generates get/set/remove/iter methods for each component type.
macro_rules! component_accessors {
    ($field:ident, $t:ty, $get:ident, $set:ident, $remove:ident, $iter:ident) => {
        #[doc = concat!("Get the `", stringify!($t), "` component of an entity.")]
        #[must_use]
        pub fn $get(&self, entity: Entity) -> Option<$t> {
            self.$field.get(entity)
        }

        #[doc = concat!("Set the `", stringify!($t), "` component of an entity.")]
        pub fn $set(&mut self, entity: Entity, value: impl Into<$t>) {
            self.$field.insert(entity, value.into());
        }

        #[doc = concat!("Remove the `", stringify!($t), "` component of an entity.")]
        pub fn $remove(&mut self, entity: Entity) -> Option<$t> {
            self.$field.remove(entity)
        }

        #[doc = concat!("Iterate all entities with the `", stringify!($t), "` component.")]
        pub fn $iter(&self) -> impl Iterator<Item = (Entity, &$t)> {
            self.$field.iter()
        }
    };
}

/// ECS World — the container for all entities and components.
///
/// # Internal layout
///
/// - `generations`: generation version number per slot (incremented on despawn)
/// - `free_list`: reusable free slots (FIFO)
/// - 10 component sparse sets + the Zones table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    /// Active aura source index (see [`AuraIndex`]) — incrementally maintained by all relevant mutating methods.
    aura_index: AuraIndex,
    /// Generation version number per slot, used to detect stale Entity handles
    generations: Vec<u32>,
    /// Reusable free slot indices
    free_list: Vec<u32>,
    /// Health component storage
    health: SparseSet<Health>,
    /// Attack component storage
    attack: SparseSet<Attack>,
    /// Cost component storage
    cost: SparseSet<Cost>,
    /// CardType component storage
    card_type: SparseSet<CardType>,
    /// Race component storage (fidelity-debt W1)
    /// Race storage — a minion may carry several tribes (Core Set W1:
    /// Mythical Terror is Demon + Beast; the classic single-tribe cards
    /// hold exactly one entry).
    race: SparseSet<Vec<Race>>,
    /// Zone component storage (the entity's current zone)
    zone_comp: SparseSet<Zone>,
    /// PlayerId component storage
    player_comp: SparseSet<PlayerId>,
    /// AttacksUsed component storage
    attacks_used: SparseSet<AttacksUsed>,
    /// Battlecry component storage
    battlecry: SparseSet<Battlecry>,
    /// Deathrattle component storage
    deathrattle: SparseSet<Deathrattle>,
    /// Taunt component storage
    taunt: SparseSet<Taunt>,
    /// Durability component storage (weapon durability)
    durability: SparseSet<Durability>,
    /// Armor component storage (hero armor)
    armor: SparseSet<Armor>,
    /// HeroPowerDef component storage (hero power definition)
    hero_power: SparseSet<HeroPowerDef>,
    /// HeroPowerUsed component storage (whether the power was used this turn)
    hero_power_used: SparseSet<HeroPowerUsed>,
    /// Aura component storage (aura effects)
    aura: SparseSet<Aura>,
    /// Secret component storage (secrets)
    secret: SparseSet<Secret>,
    /// DivineShield component storage (divine shields)
    divine_shield: SparseSet<DivineShield>,
    /// Windfury component storage (windfury)
    windfury: SparseSet<Windfury>,
    /// Charge component storage (charge)
    charge: SparseSet<Charge>,
    /// SpellDamage component storage (spell damage)
    spell_damage: SparseSet<SpellDamage>,
    /// Freeze component storage (freeze)
    freeze: SparseSet<Freeze>,
    /// CantAttack component storage (cannot attack)
    cant_attack: SparseSet<CantAttack>,
    /// Trigger component storage — registered per-entity triggers (roadmap G2)
    trigger: SparseSet<Trigger>,
    /// ChooseOneEffect component storage (Choose One effects)
    choose_one_effect: SparseSet<ChooseOneEffect>,
    /// ComboEffect component storage (combo effects)
    combo_effect: SparseSet<ComboEffect>,
    /// AttackEqualsHealth component storage (Lightspawn)
    attack_equals_health: SparseSet<AttackEqualsHealth>,
    /// Enchantment component storage — stat modifiers (roadmap G4)
    enchantments: SparseSet<Vec<Enchantment>>,
    /// Cost modifier storage — set/floor cost modifiers (roadmap G5)
    cost_modifier: SparseSet<Vec<CostModifier>>,
    /// Damage component storage — accumulated damage (roadmap G4)
    damage: SparseSet<Damage>,
    /// CardId component storage (original card definition ID)
    card_id: SparseSet<CardId>,
    /// Poison component storage (poison)
    poison: SparseSet<Poison>,
    /// Rush component storage (Core Set W1)
    rush: SparseSet<Rush>,
    /// Lifesteal component storage (Core Set W1)
    lifesteal: SparseSet<Lifesteal>,
    /// Reborn component storage (Core Set W1)
    reborn: SparseSet<Reborn>,
    /// SummonedThisTurn component storage (Core Set W1)
    summoned_this_turn: SparseSet<SummonedThisTurn>,
    /// Enrage component storage — the damaged-only conditional bonus
    enrage: SparseSet<Enrage>,
    /// Stealth component storage (stealth)
    stealth: SparseSet<Stealth>,
    /// Elusive component storage (elusive; M5)
    elusive: SparseSet<Elusive>,
    /// Immune component storage (immune)
    immune: SparseSet<Immune>,
    /// Overload component storage (overload marker)
    overload: SparseSet<Overload>,
    /// Zone table — ordered entity lists per Zone
    zones: Zones,
}

/// Aura source index — buckets live battlefield aura sources by (owner, effect kind).
///
/// Aura applicability depends on three volatile properties of the source (alive, on the
/// battlefield, owner) plus the effect kind. Scanning all `Aura` components per query is
/// O(entities × auras); this index limits each query to the buckets that can affect it.
///
/// The index is incrementally maintained by World's relevant mutating methods
/// (`set_aura`/`remove_aura`/`set_zone`/`set_player`/`despawn`); the query path is
/// read-only and lock-free.
///
/// Note: the index trusts only the `zone` component (consistent with query semantics),
/// not the `Zones` table, so direct zone-table manipulation via `zones_mut()` cannot
/// cause the index and queries to diverge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct AuraIndex {
    /// Aura sources affecting attack (bucketed by owner)
    attack: [Vec<(Entity, Aura)>; 2],
    /// Aura sources affecting health (bucketed by owner)
    health: [Vec<(Entity, Aura)>; 2],
    /// Aura sources reducing cost (bucketed by owner)
    cost: [Vec<(Entity, Aura)>; 2],
}

impl AuraIndex {
    /// Create an empty index.
    #[must_use]
    const fn new() -> Self {
        Self {
            attack: [const { Vec::new() }, const { Vec::new() }],
            health: [const { Vec::new() }, const { Vec::new() }],
            cost: [const { Vec::new() }, const { Vec::new() }],
        }
    }

    /// Remove an entity from all buckets.
    fn remove_entity(&mut self, entity: Entity) {
        for player in 0..PlayerId::COUNT {
            self.attack[player].retain(|(e, _)| *e != entity);
            self.health[player].retain(|(e, _)| *e != entity);
            self.cost[player].retain(|(e, _)| *e != entity);
        }
    }

    /// Add an entity to the bucket matching its effect kind.
    fn add_entity(&mut self, entity: Entity, aura: Aura, owner: PlayerId) {
        use crate::core::component::AuraEffect;
        let oi = owner.index();
        match aura.effect {
            AuraEffect::GainStats { attack, health } => {
                if attack != 0 {
                    self.attack[oi].push((entity, aura));
                }
                if health != 0 {
                    self.health[oi].push((entity, aura));
                }
            }
            AuraEffect::GainAttack(_) | AuraEffect::GrantCharge | AuraEffect::ChargeWithWeapon => {
                self.attack[oi].push((entity, aura))
            }
            AuraEffect::GainHealth(_) => self.health[oi].push((entity, aura)),
            AuraEffect::ReduceSpellCost(_)
            | AuraEffect::ReduceMinionCost { .. }
            | AuraEffect::FirstMinionDiscount { .. }
            | AuraEffect::IncreaseMinionCost { .. }
            | AuraEffect::IncreaseMinionCostFriendly { .. } => self.cost[oi].push((entity, aura)),
        }
    }
}

impl World {
    /// Create an empty world.
    #[must_use]
    pub fn new() -> Self {
        Self {
            aura_index: AuraIndex::new(),
            generations: Vec::new(),
            free_list: Vec::new(),
            health: SparseSet::new(),
            attack: SparseSet::new(),
            cost: SparseSet::new(),
            card_type: SparseSet::new(),
            race: SparseSet::new(),
            zone_comp: SparseSet::new(),
            player_comp: SparseSet::new(),
            attacks_used: SparseSet::new(),
            battlecry: SparseSet::new(),
            deathrattle: SparseSet::new(),
            taunt: SparseSet::new(),
            durability: SparseSet::new(),
            armor: SparseSet::new(),
            hero_power: SparseSet::new(),
            hero_power_used: SparseSet::new(),
            aura: SparseSet::new(),
            secret: SparseSet::new(),
            divine_shield: SparseSet::new(),
            windfury: SparseSet::new(),
            charge: SparseSet::new(),
            spell_damage: SparseSet::new(),
            freeze: SparseSet::new(),
            cant_attack: SparseSet::new(),
            trigger: SparseSet::new(),
            choose_one_effect: SparseSet::new(),
            combo_effect: SparseSet::new(),
            attack_equals_health: SparseSet::new(),
            enchantments: SparseSet::new(),
            damage: SparseSet::new(),
            cost_modifier: SparseSet::new(),
            card_id: SparseSet::new(),
            poison: SparseSet::new(),
            rush: SparseSet::new(),
            lifesteal: SparseSet::new(),
            reborn: SparseSet::new(),
            summoned_this_turn: SparseSet::new(),
            enrage: SparseSet::new(),
            stealth: SparseSet::new(),
            elusive: SparseSet::new(),
            immune: SparseSet::new(),
            overload: SparseSet::new(),
            zones: Zones::new(),
        }
    }

    /// Spawn a new entity and return its handle.
    ///
    /// Reuses a free slot when available; otherwise grows the arrays.
    pub fn spawn(&mut self) -> Entity {
        if let Some(index) = self.free_list.pop() {
            let generation = self.generations[index as usize];
            Entity::new(index, generation)
        } else {
            let index = self.generations.len() as u32;
            self.generations.push(0);
            Entity::new(index, 0)
        }
    }

    /// Check whether the entity is still alive (generation matches).
    #[must_use]
    pub fn is_alive(&self, entity: Entity) -> bool {
        let idx = entity.index as usize;
        idx < self.generations.len() && self.generations[idx] == entity.generation
    }

    /// Destroy an entity: clear all components, remove it from all zones, bump the generation, and return the slot.
    ///
    /// In Phase 1 despawn is only used for cleanup (tests, etc.).
    /// In-game deaths use `move_to_zone(entity, Zone::Graveyard)` instead of despawn.
    pub fn despawn(&mut self, entity: Entity) {
        if !self.is_alive(entity) {
            return;
        }
        let idx = entity.index as usize;
        // Remove from all zones
        self.zones.remove_from_all(entity);
        // Clear all components
        self.health.remove(entity);
        self.attack.remove(entity);
        self.cost.remove(entity);
        self.card_type.remove(entity);
        self.race.remove(entity);
        self.zone_comp.remove(entity);
        self.player_comp.remove(entity);
        self.attacks_used.remove(entity);
        self.battlecry.remove(entity);
        self.deathrattle.remove(entity);
        self.taunt.remove(entity);
        self.durability.remove(entity);
        self.armor.remove(entity);
        self.hero_power.remove(entity);
        self.hero_power_used.remove(entity);
        self.remove_aura(entity);
        self.secret.remove(entity);
        self.divine_shield.remove(entity);
        self.windfury.remove(entity);
        self.charge.remove(entity);
        self.spell_damage.remove(entity);
        self.freeze.remove(entity);
        self.cant_attack.remove(entity);
        self.trigger.remove(entity);
        self.choose_one_effect.remove(entity);
        self.combo_effect.remove(entity);
        self.attack_equals_health.remove(entity);
        self.enchantments.remove(entity);
        self.damage.remove(entity);
        self.cost_modifier.remove(entity);
        self.card_id.remove(entity);
        self.poison.remove(entity);
        self.rush.remove(entity);
        self.lifesteal.remove(entity);
        self.reborn.remove(entity);
        self.summoned_this_turn.remove(entity);
        self.enrage.remove(entity);
        self.stealth.remove(entity);
        self.elusive.remove(entity);
        self.immune.remove(entity);
        self.overload.remove(entity);
        // Bump the generation
        self.generations[idx] = self.generations[idx].wrapping_add(1);
        // Return the slot
        self.free_list.push(entity.index);
    }

    /// Move an entity from one zone to another.
    ///
    /// This is the **only entry point** for zone transfers, keeping the Zone component and the Zones table in sync.
    ///
    /// # Errors
    ///
    /// - `MoveError::EntityGone` — the entity has been destroyed
    /// - `MoveError::MissingPlayer` — the PlayerId component is missing, so the owner cannot be determined
    /// - `MoveError::MissingZone` — the current Zone component is missing (inconsistent state)
    pub fn move_to_zone(&mut self, entity: Entity, target: Zone) -> Result<(), MoveError> {
        if !self.is_alive(entity) {
            return Err(MoveError::EntityGone);
        }
        let player = self.player(entity).ok_or(MoveError::MissingPlayer)?;
        let current = self.zone(entity).ok_or(MoveError::MissingZone)?;

        // Remove from the old zone
        self.zones.remove(current, player, entity);
        // Insert into the new zone
        self.zones.insert(target, player, entity);
        // Update the Zone component
        self.set_zone(entity, target);

        // Roadmap G4 — leaving the battlefield removes enchantments and damage
        // (bounced minions return at full health, unbuffed; cost modifiers are
        // also removed, matching HS — bounce effects like Shadowstep re-apply
        // their own cost enchantment).
        if current == Zone::Play && target != Zone::Play {
            self.remove_enchantments(entity);
            self.remove_damage(entity);
        }

        Ok(())
    }

    /// Get a read-only reference to the Zones table.
    #[must_use]
    pub fn zones(&self) -> &Zones {
        &self.zones
    }

    /// Get a mutable reference to the Zones table (for tests/GameBuilder to manipulate zones directly).
    ///
    /// ⚠️ Directly manipulating the Zones table requires updating the Zone components in
    /// parallel, or the state becomes inconsistent. Prefer `move_to_zone`.
    ///
    /// The aura index trusts only the `Zone` component (consistent with query semantics),
    /// not this table, so direct zone-table manipulation cannot cause the index and queries to diverge.
    pub fn zones_mut(&mut self) -> &mut Zones {
        &mut self.zones
    }

    // Generate accessor methods for each component type
    component_accessors!(
        health,
        Health,
        health,
        set_health,
        remove_health,
        iter_health
    );
    component_accessors!(
        attack,
        Attack,
        attack,
        set_attack,
        remove_attack,
        iter_attack
    );
    component_accessors!(cost, Cost, cost, set_cost, remove_cost, iter_cost);
    component_accessors!(
        card_type,
        CardType,
        card_type,
        set_card_type,
        remove_card_type,
        iter_card_type
    );
    /// Get an entity's `Zone` component.
    #[must_use]
    pub fn zone(&self, entity: Entity) -> Option<Zone> {
        self.zone_comp.get(entity)
    }

    /// Set an entity's `Zone` component (may change aura applicability; the index is incrementally maintained).
    pub fn set_zone(&mut self, entity: Entity, value: impl Into<Zone>) {
        let value = value.into();
        // An aura source entering or leaving the battlefield changes its "active" state; keep the index in sync
        if self.aura.contains(entity) {
            let was_active = self.zone(entity) == Some(Zone::Play);
            let now_active = value == Zone::Play;
            if was_active && !now_active {
                self.aura_index.remove_entity(entity);
            } else if !was_active && now_active {
                if let (Some(aura), Some(player)) = (self.aura.get(entity), self.player(entity)) {
                    self.aura_index.add_entity(entity, aura, player);
                }
            }
        }
        self.zone_comp.insert(entity, value);
    }

    /// Remove an entity's `Zone` component (the aura index loses the entity's active-state information).
    pub fn remove_zone(&mut self, entity: Entity) -> Option<Zone> {
        let removed = self.zone_comp.remove(entity);
        if removed.is_some() && self.aura.contains(entity) {
            self.aura_index.remove_entity(entity);
        }
        removed
    }

    /// Iterate over all entities with a `Zone` component.
    pub fn iter_zone(&self) -> impl Iterator<Item = (Entity, &Zone)> {
        self.zone_comp.iter()
    }

    /// Get an entity's `PlayerId` component.
    #[must_use]
    pub fn player(&self, entity: Entity) -> Option<PlayerId> {
        self.player_comp.get(entity)
    }

    /// Set an entity's `PlayerId` component (an owner change rebuckets auras; the index is incrementally maintained).
    pub fn set_player(&mut self, entity: Entity, value: impl Into<PlayerId>) {
        let value = value.into();
        // An active aura source switching sides: remove it from its old bucket first, then re-add it under the new owner
        if self.aura.contains(entity) && self.zone(entity) == Some(Zone::Play) {
            self.aura_index.remove_entity(entity);
        }
        self.player_comp.insert(entity, value);
        if let Some(aura) = self.aura.get(entity) {
            if self.zone(entity) == Some(Zone::Play) {
                self.aura_index.add_entity(entity, aura, value);
            }
        }
    }

    /// Remove an entity's `PlayerId` component (the aura index loses the entity's active-state information).
    pub fn remove_player(&mut self, entity: Entity) -> Option<PlayerId> {
        let removed = self.player_comp.remove(entity);
        if removed.is_some() && self.aura.contains(entity) && self.zone(entity) == Some(Zone::Play)
        {
            self.aura_index.remove_entity(entity);
        }
        removed
    }

    /// Iterate over all entities with a `PlayerId` component.
    pub fn iter_player(&self) -> impl Iterator<Item = (Entity, &PlayerId)> {
        self.player_comp.iter()
    }
    component_accessors!(
        attacks_used,
        AttacksUsed,
        attacks_used,
        set_attacks_used,
        remove_attacks_used,
        iter_attacks_used
    );
    component_accessors!(
        battlecry,
        Battlecry,
        battlecry,
        set_battlecry,
        remove_battlecry,
        iter_battlecry
    );
    component_accessors!(
        deathrattle,
        Deathrattle,
        deathrattle,
        set_deathrattle,
        remove_deathrattle,
        iter_deathrattle
    );
    component_accessors!(taunt, Taunt, taunt, set_taunt, remove_taunt, iter_taunt);
    component_accessors!(
        durability,
        Durability,
        durability,
        set_durability,
        remove_durability,
        iter_durability
    );
    component_accessors!(armor, Armor, armor, set_armor, remove_armor, iter_armor);
    component_accessors!(
        hero_power,
        HeroPowerDef,
        hero_power,
        set_hero_power,
        remove_hero_power,
        iter_hero_power
    );
    component_accessors!(
        hero_power_used,
        HeroPowerUsed,
        hero_power_used,
        set_hero_power_used,
        remove_hero_power_used,
        iter_hero_power_used
    );
    /// Get an entity's `Aura` component.
    #[must_use]
    pub fn aura(&self, entity: Entity) -> Option<Aura> {
        self.aura.get(entity)
    }

    /// Set an entity's `Aura` component (the aura source set may change; the index is incrementally maintained).
    pub fn set_aura(&mut self, entity: Entity, value: impl Into<Aura>) {
        let value = value.into();
        // An active aura source's effect is updated: remove it from the index first, then re-add it with the new value
        if self.aura.contains(entity) && self.zone(entity) == Some(Zone::Play) {
            self.aura_index.remove_entity(entity);
        }
        self.aura.insert(entity, value);
        if let Some(player) = self.player(entity) {
            if self.zone(entity) == Some(Zone::Play) {
                self.aura_index.add_entity(entity, value, player);
            }
        }
    }

    /// Remove an entity's `Aura` component (removes the aura source from the index).
    pub fn remove_aura(&mut self, entity: Entity) -> Option<Aura> {
        let removed = self.aura.remove(entity);
        if removed.is_some() {
            self.aura_index.remove_entity(entity);
        }
        removed
    }

    /// Iterate over all entities with an `Aura` component.
    pub fn iter_aura(&self) -> impl Iterator<Item = (Entity, &Aura)> {
        self.aura.iter()
    }
    component_accessors!(
        secret,
        Secret,
        secret,
        set_secret,
        remove_secret,
        iter_secret
    );
    component_accessors!(
        divine_shield,
        DivineShield,
        divine_shield,
        set_divine_shield,
        remove_divine_shield,
        iter_divine_shield
    );
    component_accessors!(
        windfury,
        Windfury,
        windfury,
        set_windfury,
        remove_windfury,
        iter_windfury
    );
    component_accessors!(
        charge,
        Charge,
        charge,
        set_charge,
        remove_charge,
        iter_charge
    );
    component_accessors!(
        spell_damage,
        SpellDamage,
        spell_damage,
        set_spell_damage,
        remove_spell_damage,
        iter_spell_damage
    );
    component_accessors!(
        freeze,
        Freeze,
        freeze,
        set_freeze,
        remove_freeze,
        iter_freeze
    );
    component_accessors!(
        cant_attack,
        CantAttack,
        cant_attack,
        set_cant_attack,
        remove_cant_attack,
        iter_cant_attack
    );
    component_accessors!(
        trigger,
        Trigger,
        trigger,
        set_trigger,
        remove_trigger,
        iter_trigger
    );
    component_accessors!(
        choose_one_effect,
        ChooseOneEffect,
        choose_one_effect,
        set_choose_one_effect,
        remove_choose_one_effect,
        iter_choose_one_effect
    );
    component_accessors!(
        combo_effect,
        ComboEffect,
        combo_effect,
        set_combo_effect,
        remove_combo_effect,
        iter_combo_effect
    );
    component_accessors!(
        attack_equals_health,
        AttackEqualsHealth,
        attack_equals_health,
        set_attack_equals_health,
        remove_attack_equals_health,
        iter_attack_equals_health
    );
    /// Get the enchantments attached to an entity (roadmap G4).
    #[must_use]
    pub fn enchantments(&self, entity: Entity) -> Option<&[Enchantment]> {
        self.enchantments.get_ref(entity).map(Vec::as_slice)
    }

    /// Attach an enchantment to an entity.
    pub fn add_enchantment(&mut self, entity: Entity, enchantment: Enchantment) {
        let mut list = self.enchantments.get(entity).unwrap_or_default();
        list.push(enchantment);
        self.enchantments.insert(entity, list);
    }

    /// Remove enchantments for which `keep` returns `false`; drops the component
    /// when none remain. Returns whether any enchantment was removed.
    pub fn retain_enchantments(
        &mut self,
        entity: Entity,
        keep: impl FnMut(&Enchantment) -> bool,
    ) -> bool {
        let Some(mut list) = self.enchantments.get(entity) else {
            return false;
        };
        let before = list.len();
        list.retain(keep);
        let removed = list.len() != before;
        if list.is_empty() {
            self.enchantments.remove(entity);
        } else {
            self.enchantments.insert(entity, list);
        }
        removed
    }

    /// Remove all enchantments from an entity.
    pub fn remove_enchantments(&mut self, entity: Entity) {
        self.enchantments.remove(entity);
    }

    /// Get the cost modifiers on an entity (roadmap G5).
    #[must_use]
    pub fn cost_modifiers(&self, entity: Entity) -> Option<&[CostModifier]> {
        self.cost_modifier.get_ref(entity).map(Vec::as_slice)
    }

    /// Attach a cost modifier to an entity.
    pub fn add_cost_modifier(&mut self, entity: Entity, modifier: CostModifier) {
        let mut list = self.cost_modifier.get(entity).unwrap_or_default();
        list.push(modifier);
        self.cost_modifier.insert(entity, list);
    }

    /// Remove all cost modifiers from an entity.
    pub fn remove_cost_modifiers(&mut self, entity: Entity) {
        self.cost_modifier.remove(entity);
    }

    /// Iterate all entities with enchantments.
    pub fn iter_enchantments(&self) -> impl Iterator<Item = (Entity, &Vec<Enchantment>)> {
        self.enchantments.iter()
    }

    /// Get the accumulated damage on an entity.
    #[must_use]
    pub fn damage(&self, entity: Entity) -> Option<Damage> {
        self.damage.get(entity)
    }

    /// Set the accumulated damage on an entity.
    pub fn set_damage(&mut self, entity: Entity, value: impl Into<Damage>) {
        self.damage.insert(entity, value.into());
    }

    /// Clear the accumulated damage on an entity.
    pub fn remove_damage(&mut self, entity: Entity) {
        self.damage.remove(entity);
    }
    component_accessors!(
        card_id,
        CardId,
        card_id,
        set_card_id,
        remove_card_id,
        iter_card_id
    );
    component_accessors!(
        poison,
        Poison,
        poison,
        set_poison,
        remove_poison,
        iter_poison
    );
    component_accessors!(rush, Rush, rush, set_rush, remove_rush, iter_rush);
    component_accessors!(
        lifesteal,
        Lifesteal,
        lifesteal,
        set_lifesteal,
        remove_lifesteal,
        iter_lifesteal
    );
    component_accessors!(
        reborn,
        Reborn,
        reborn,
        set_reborn,
        remove_reborn,
        iter_reborn
    );
    component_accessors!(
        summoned_this_turn,
        SummonedThisTurn,
        summoned_this_turn,
        set_summoned_this_turn,
        remove_summoned_this_turn,
        iter_summoned_this_turn
    );
    component_accessors!(
        enrage,
        Enrage,
        enrage,
        set_enrage,
        remove_enrage,
        iter_enrage
    );
    component_accessors!(
        stealth,
        Stealth,
        stealth,
        set_stealth,
        remove_stealth,
        iter_stealth
    );
    component_accessors!(
        elusive,
        Elusive,
        elusive,
        set_elusive,
        remove_elusive,
        iter_elusive
    );
    component_accessors!(
        immune,
        Immune,
        immune,
        set_immune,
        remove_immune,
        iter_immune
    );
    component_accessors!(
        overload,
        Overload,
        overload,
        set_overload,
        remove_overload,
        iter_overload
    );
    /// Get the tribes of an entity (empty for tribe-less minions).
    #[must_use]
    pub fn race(&self, entity: Entity) -> Option<&[Race]> {
        self.race.get_ref(entity).map(Vec::as_slice)
    }

    /// Whether the entity carries the given tribe (any of its tribes).
    #[must_use]
    pub fn has_race(&self, entity: Entity, race: Race) -> bool {
        self.race
            .get_ref(entity)
            .is_some_and(|tribes| tribes.contains(&race))
    }

    /// Set the tribes of an entity to exactly one tribe (the primary
    /// tribe from the card definition).
    pub fn set_race(&mut self, entity: Entity, race: Race) {
        self.race.insert(entity, vec![race]);
    }

    /// Add a secondary tribe to an entity (Core Set W1 — dual-tribe cards
    /// such as Mythical Terror; no-op when the tribe is already present).
    pub fn add_race(&mut self, entity: Entity, race: Race) {
        let has = self
            .race
            .get_ref(entity)
            .is_some_and(|tribes| tribes.contains(&race));
        if has {
            return;
        }
        let mut tribes = self.race.get(entity).unwrap_or_default();
        tribes.push(race);
        self.race.insert(entity, tribes);
    }

    /// Remove all tribes from an entity.
    pub fn remove_race(&mut self, entity: Entity) -> Option<Vec<Race>> {
        self.race.remove(entity)
    }

    /// Iterate all entities with tribes (entity, tribes).
    pub fn iter_race(&self) -> impl Iterator<Item = (Entity, &Vec<Race>)> {
        self.race.iter()
    }
    /// Whether an entity effectively has Charge — its base Charge component or
    /// an applying Charge aura (Tundra Rhino — your Beasts have Charge).
    #[must_use]
    pub fn effective_charge(&self, entity: Entity) -> bool {
        if self.charge(entity).is_some() {
            return true;
        }
        let Some(player) = self.player(entity) else {
            return false;
        };
        for owner in [player, player.opponent()] {
            for (source, aura) in &self.aura_index.attack[owner.index()] {
                let grants = match aura.effect {
                    crate::core::component::AuraEffect::GrantCharge => true,
                    // Southsea Deckhand — Charge while the owner has a weapon
                    // (the weapon entity sits in the owner's Play zone)
                    crate::core::component::AuraEffect::ChargeWithWeapon => {
                        self.player(*source).is_some_and(|p| {
                            self.zones()
                                .iter(crate::core::zone::Zone::Play, p)
                                .any(|e| {
                                    self.card_type(e)
                                        == Some(crate::core::component::CardType::Weapon)
                                })
                        })
                    }
                    _ => false,
                };
                if grants && aura_applies_to(aura, *source, owner, entity, player, self) {
                    return true;
                }
            }
        }
        false
    }

    /// Whether an entity currently sits below its maximum Health.
    ///
    /// This is the Enrage condition. Accumulated damage is the only thing that
    /// can put a character below its maximum (roadmap G4 — buffs raise base
    /// Health rather than lowering damage), so a non-zero `Damage` component is
    /// exactly "damaged".
    #[must_use]
    pub fn is_damaged(&self, entity: Entity) -> bool {
        self.damage(entity).is_some_and(|d| d.0 > 0)
    }

    /// The Enrage bonus this minion currently grants itself — zero unless it
    /// has an `Enrage` component *and* is damaged.
    #[must_use]
    fn active_enrage(&self, entity: Entity) -> Option<crate::core::component::Enrage> {
        self.enrage(entity).filter(|_| self.is_damaged(entity))
    }

    /// Get the maximum number of attacks an entity can make per turn.
    ///
    /// Raging Worgen's Windfury is part of its Enrage, so it applies only while
    /// the worgen is damaged.
    #[must_use]
    pub fn max_attacks(&self, entity: Entity) -> u8 {
        let enraged_windfury = self.active_enrage(entity).is_some_and(|e| e.windfury);
        if self.windfury(entity).is_some() || enraged_windfury {
            2
        } else {
            1
        }
    }

    /// Get the total friendly spell damage on the board.
    #[must_use]
    pub fn total_spell_damage(&self, player: PlayerId) -> i32 {
        use crate::core::zone::Zone;
        let mut total = 0i32;
        for (e, sd) in self.iter_spell_damage() {
            if self.is_alive(e)
                && self.zone(e) == Some(Zone::Play)
                && self.player(e) == Some(player)
            {
                total += sd.0;
            }
        }
        total
    }

    /// Get an entity's effective attack (base attack + all aura bonuses).
    ///
    /// Uses the aura source index to scan only the auras that can affect attack
    /// (the friendly and enemy attack buckets) instead of iterating over all `Aura` components.
    ///
    /// Enrage is resolved here rather than being written into an enchantment,
    /// which is what makes it non-stacking and makes it vanish the instant the
    /// minion is healed to full.
    #[must_use]
    pub fn effective_attack(&self, entity: Entity) -> Option<Attack> {
        let base = self.attack(entity)?;
        let player = self.player(entity)?;

        // Enchantment deltas (roadmap G4)
        let mut bonus = self
            .enchantments(entity)
            .map_or(0, |list| list.iter().map(|e| e.attack).sum::<i32>());
        // Friendly auras affect friendly minions; enemy auras affect friendly minions via AllEnemyMinions
        for owner in [player, player.opponent()] {
            for (source, aura) in &self.aura_index.attack[owner.index()] {
                if aura_applies_to(aura, *source, owner, entity, player, self) {
                    bonus += aura_attack_bonus(aura.effect);
                }
            }
        }

        // This minion's own Enrage (Amani Berserker, Raging Worgen, Tauren
        // Warrior, Angry Chicken, Grommash Hellscream)
        if let Some(enrage) = self.active_enrage(entity) {
            bonus += enrage.attack;
        }

        // Spiteful Smith — a damaged Enrage minion buffs its owner's weapon.
        // The bonus lives on the smith, so it is read from the weapon's side:
        // auras never apply to weapons (`aura_applies_to` rejects non-minions),
        // which is why this cannot ride the aura index.
        if self.card_type(entity) == Some(CardType::Weapon) {
            for (source, enrage) in self.iter_enrage() {
                if enrage.weapon_attack != 0
                    && self.player(source) == Some(player)
                    && self.zone(source) == Some(crate::core::zone::Zone::Play)
                    && self.is_alive(source)
                    && self.is_damaged(source)
                {
                    bonus += enrage.weapon_attack;
                }
            }
        }

        Some(Attack(base.0 + bonus))
    }

    /// Get an entity's effective health (base health + all aura bonuses).
    ///
    /// Uses the aura source index to scan only the auras that can affect health
    /// (the friendly and enemy health buckets).
    #[must_use]
    pub fn effective_health(&self, entity: Entity) -> Option<Health> {
        let base = self.health(entity)?;
        let player = self.player(entity)?;

        // Enchantment health deltas minus accumulated damage (roadmap G4)
        let mut bonus = self
            .enchantments(entity)
            .map_or(0, |list| list.iter().map(|e| e.health).sum::<i32>())
            - self.damage(entity).map_or(0, |d| d.0);
        for owner in [player, player.opponent()] {
            for (source, aura) in &self.aura_index.health[owner.index()] {
                if aura_applies_to(aura, *source, owner, entity, player, self) {
                    bonus += aura_health_bonus(aura.effect);
                }
            }
        }

        Some(Health(base.0 + bonus))
    }

    /// Get an entity's effective mana cost (base cost − cost-reduction auras).
    ///
    /// Uses the aura source index to scan only the friendly cost bucket:
    /// - `ReduceSpellCost` applies to friendly spells in hand (Sorcerer's Apprentice)
    /// - `ReduceMinionCost` applies to friendly minions in hand, with a cost floor
    ///   (Summoning Portal — at least 1 mana)
    #[must_use]
    pub fn effective_cost(&self, entity: Entity) -> Option<Cost> {
        use crate::core::component::AuraEffect;

        let base = self.cost(entity)?;
        let player = self.player(entity)?;
        let card_type = self.card_type(entity)?;
        let in_hand = self.zone(entity) == Some(Zone::Hand);

        // Cost auras only affect the aura owner's own hand
        // Enchantment cost deltas (roadmap G4) — cost modifiers persist across zones
        let mut cost = base.0
            + self
                .enchantments(entity)
                .map_or(0, |l| l.iter().map(|e| e.cost).sum::<i32>());

        // Cost modifier stack (roadmap G5): set-to-value overrides the composed
        // value; floor modifiers raise it.
        for modifier in self.cost_modifiers(entity).into_iter().flatten() {
            match modifier.kind {
                CostModifierKind::Set(value) => cost = value,
                CostModifierKind::Min(floor) => cost = cost.max(floor),
            }
        }

        let mut reduction = 0i32;
        let mut increase = 0i32;
        let mut min_cost = 0i32;
        // Friendly reductions come from the owner's bucket; global increases
        // (Mana Wraith — ALL minions) from both buckets. The friendly-scoped
        // variants only apply to the aura owner's own hand.
        for owner in [player, player.opponent()] {
            for (_, aura) in &self.aura_index.cost[owner.index()] {
                match aura.effect {
                    AuraEffect::ReduceSpellCost(amount)
                        if owner == player && in_hand && card_type == CardType::Spell =>
                    {
                        reduction += amount;
                    }
                    AuraEffect::ReduceMinionCost { amount, min }
                        if owner == player && in_hand && card_type == CardType::Minion =>
                    {
                        reduction += amount;
                        min_cost = min_cost.max(min);
                    }
                    AuraEffect::IncreaseMinionCost { amount }
                        if in_hand && card_type == CardType::Minion =>
                    {
                        increase += amount;
                    }
                    AuraEffect::IncreaseMinionCostFriendly { amount }
                        if owner == player && in_hand && card_type == CardType::Minion =>
                    {
                        increase += amount;
                    }
                    _ => {}
                }
            }
        }
        // The cost can never go below 0 (and never below the aura's min floor)
        Some(Cost((cost + increase - reduction).max(min_cost.max(0))))
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Aura helper functions
// ============================================================

/// Check whether an aura effect applies to the target entity.
fn aura_applies_to(
    aura: &Aura,
    aura_source: Entity,
    aura_player: PlayerId,
    target: Entity,
    target_player: PlayerId,
    world: &World,
) -> bool {
    use crate::core::component::{AuraTarget, CardType};

    // The target must be a living minion
    if world.card_type(target) != Some(CardType::Minion) {
        return false;
    }
    if !world.is_alive(target) {
        return false;
    }

    match aura.target {
        AuraTarget::AllFriendlyMinions => target_player == aura_player,
        AuraTarget::OtherFriendlyMinions => target_player == aura_player && target != aura_source,
        AuraTarget::FriendlyRace(race) => {
            target_player == aura_player && world.has_race(target, race)
        }
        AuraTarget::OtherFriendlyRace(race) => {
            target_player == aura_player && target != aura_source && world.has_race(target, race)
        }
        AuraTarget::AdjacentMinions => {
            if target_player != aura_player || target == aura_source {
                return false;
            }
            is_adjacent(aura_source, target, aura_player, world)
        }
        AuraTarget::AllEnemyMinions => target_player != aura_player,
    }
}

/// Check whether two entities are adjacent on the battlefield.
fn is_adjacent(source: Entity, target: Entity, player: PlayerId, world: &World) -> bool {
    use crate::core::component::CardType;
    use crate::core::zone::Zone;

    let minions: Vec<Entity> = world
        .zones()
        .iter(Zone::Play, player)
        .filter(|&e| world.card_type(e) == Some(CardType::Minion) && world.is_alive(e))
        .collect();

    let source_pos = minions.iter().position(|&e| e == source);
    let target_pos = minions.iter().position(|&e| e == target);

    match (source_pos, target_pos) {
        (Some(s), Some(t)) => {
            // Adjacent = position difference of 1
            (s as isize - t as isize).unsigned_abs() == 1
        }
        _ => false,
    }
}

/// Returns the attack bonus of an aura effect.
const fn aura_attack_bonus(effect: crate::core::component::AuraEffect) -> i32 {
    use crate::core::component::AuraEffect;
    match effect {
        AuraEffect::GainStats { attack, .. } => attack,
        AuraEffect::GainAttack(a) => a,
        AuraEffect::GainHealth(_) => 0,
        AuraEffect::ReduceSpellCost(_) => 0,
        AuraEffect::ReduceMinionCost { .. } => 0,
        AuraEffect::GrantCharge => 0,
        AuraEffect::FirstMinionDiscount { .. } => 0,
        AuraEffect::IncreaseMinionCost { .. } => 0,
        AuraEffect::IncreaseMinionCostFriendly { .. } => 0,
        AuraEffect::ChargeWithWeapon => 0,
    }
}

/// Returns the health bonus of an aura effect.
const fn aura_health_bonus(effect: crate::core::component::AuraEffect) -> i32 {
    use crate::core::component::AuraEffect;
    match effect {
        AuraEffect::GainStats { health, .. } => health,
        AuraEffect::GainAttack(_) => 0,
        AuraEffect::GainHealth(h) => h,
        AuraEffect::ReduceSpellCost(_) => 0,
        AuraEffect::ReduceMinionCost { .. } => 0,
        AuraEffect::GrantCharge => 0,
        AuraEffect::FirstMinionDiscount { .. } => 0,
        AuraEffect::IncreaseMinionCost { .. } => 0,
        AuraEffect::IncreaseMinionCostFriendly { .. } => 0,
        AuraEffect::ChargeWithWeapon => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::{AuraEffect, AuraTarget, CardType};

    /// Spawn a minion on the battlefield.
    fn spawn_play_minion(world: &mut World, player: PlayerId, attack: i32, health: i32) -> Entity {
        let e = world.spawn();
        world.set_card_type(e, CardType::Minion);
        world.set_attack(e, Attack(attack));
        world.set_health(e, Health(health));
        world.set_cost(e, Cost(0));
        world.set_player(e, player);
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, player, e);
        e
    }

    /// Spawn an aura source on the battlefield.
    fn spawn_play_aura(
        world: &mut World,
        player: PlayerId,
        effect: AuraEffect,
        target: AuraTarget,
    ) -> Entity {
        let e = spawn_play_minion(world, player, 1, 1);
        world.set_aura(e, Aura { effect, target });
        e
    }

    /// Brute-force rebuilds the index from authoritative data (the aura component set plus
    /// alive/battlefield/owner checks), to verify that incremental maintenance matches "rebuild" semantics exactly.
    fn brute_force_aura_index(world: &World) -> AuraIndex {
        let mut idx = AuraIndex {
            attack: [Vec::new(), Vec::new()],
            health: [Vec::new(), Vec::new()],
            cost: [Vec::new(), Vec::new()],
        };
        for (source, aura) in world.iter_aura() {
            if !world.is_alive(source) || world.zone(source) != Some(Zone::Play) {
                continue;
            }
            let Some(owner) = world.player(source) else {
                continue;
            };
            let oi = owner.index();
            match aura.effect {
                AuraEffect::GainStats { attack, health } => {
                    if attack != 0 {
                        idx.attack[oi].push((source, *aura));
                    }
                    if health != 0 {
                        idx.health[oi].push((source, *aura));
                    }
                }
                AuraEffect::GainAttack(_)
                | AuraEffect::GrantCharge
                | AuraEffect::ChargeWithWeapon => idx.attack[oi].push((source, *aura)),
                AuraEffect::GainHealth(_) => idx.health[oi].push((source, *aura)),
                AuraEffect::ReduceSpellCost(_)
                | AuraEffect::ReduceMinionCost { .. }
                | AuraEffect::FirstMinionDiscount { .. }
                | AuraEffect::IncreaseMinionCost { .. }
                | AuraEffect::IncreaseMinionCostFriendly { .. } => {
                    idx.cost[oi].push((source, *aura))
                }
            }
        }
        idx
    }

    /// Assert that the incrementally maintained index matches the brute-force rebuild (ignoring intra-bucket order).
    fn assert_index_matches(world: &World) {
        let mut actual = world.aura_index.clone();
        let mut expected = brute_force_aura_index(world);
        for player in 0..PlayerId::COUNT {
            actual.attack[player].sort_by_key(|(e, _)| e.index);
            actual.health[player].sort_by_key(|(e, _)| e.index);
            actual.cost[player].sort_by_key(|(e, _)| e.index);
            expected.attack[player].sort_by_key(|(e, _)| e.index);
            expected.health[player].sort_by_key(|(e, _)| e.index);
            expected.cost[player].sort_by_key(|(e, _)| e.index);
        }
        assert_eq!(
            actual, expected,
            "aura index diverged from brute-force rebuild"
        );
    }

    #[test]
    fn aura_index_stays_consistent_through_mutations() {
        let mut world = World::new();
        let attack_aura = Aura {
            effect: AuraEffect::GainAttack(1),
            target: AuraTarget::AllFriendlyMinions,
        };

        // An aura card in hand (has the Aura component but is not active)
        let hand_aura = world.spawn();
        world.set_card_type(hand_aura, CardType::Minion);
        world.set_attack(hand_aura, Attack(1));
        world.set_health(hand_aura, Health(1));
        world.set_cost(hand_aura, Cost(0));
        world.set_player(hand_aura, PlayerId::Player1);
        world.set_zone(hand_aura, Zone::Hand);
        world
            .zones_mut()
            .insert(Zone::Hand, PlayerId::Player1, hand_aura);
        world.set_aura(hand_aura, attack_aura);
        assert_index_matches(&world);

        // Summoned to the battlefield → becomes active
        world.move_to_zone(hand_aura, Zone::Play).unwrap();
        assert_index_matches(&world);

        // Update the aura value while active
        world.set_aura(
            hand_aura,
            Aura {
                effect: AuraEffect::GainAttack(2),
                target: AuraTarget::AllFriendlyMinions,
            },
        );
        assert_index_matches(&world);

        // Mind Control: switches sides
        world.set_player(hand_aura, PlayerId::Player2);
        assert_index_matches(&world);

        // The aura is overwritten with GainStats (goes into two buckets)
        world.set_aura(
            hand_aura,
            Aura {
                effect: AuraEffect::GainStats {
                    attack: 1,
                    health: 1,
                },
                target: AuraTarget::AllFriendlyMinions,
            },
        );
        assert_index_matches(&world);

        // Remove the aura
        world.remove_aura(hand_aura);
        assert_index_matches(&world);

        // Re-attach the aura, then kill it (leaves the battlefield)
        world.set_aura(hand_aura, attack_aura);
        assert_index_matches(&world);
        world.move_to_zone(hand_aura, Zone::Graveyard).unwrap();
        assert_index_matches(&world);

        // Cost aura source + despawn
        let cost_aura = spawn_play_aura(
            &mut world,
            PlayerId::Player1,
            AuraEffect::ReduceSpellCost(1),
            AuraTarget::AllFriendlyMinions,
        );
        assert_index_matches(&world);
        world.despawn(cost_aura);
        assert_index_matches(&world);
    }

    #[test]
    fn aura_index_friendly_buff() {
        let mut world = World::new();
        let source = spawn_play_aura(
            &mut world,
            PlayerId::Player1,
            AuraEffect::GainAttack(1),
            AuraTarget::AllFriendlyMinions,
        );
        let target = spawn_play_minion(&mut world, PlayerId::Player1, 2, 3);

        assert_eq!(world.effective_attack(target), Some(Attack(3)));
        assert_eq!(world.effective_attack(source), Some(Attack(2)));
        // Health is not affected by attack auras
        assert_eq!(world.effective_health(target), Some(Health(3)));
    }

    #[test]
    fn aura_in_hand_does_not_apply() {
        let mut world = World::new();
        let e = world.spawn();
        world.set_card_type(e, CardType::Minion);
        world.set_attack(e, Attack(1));
        world.set_health(e, Health(1));
        world.set_cost(e, Cost(0));
        world.set_player(e, PlayerId::Player1);
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, PlayerId::Player1, e);
        world.set_aura(
            e,
            Aura {
                effect: AuraEffect::GainAttack(1),
                target: AuraTarget::AllFriendlyMinions,
            },
        );

        let target = spawn_play_minion(&mut world, PlayerId::Player1, 2, 3);
        assert_eq!(world.effective_attack(target), Some(Attack(2)));
    }

    #[test]
    fn aura_removed_invalidates_cache() {
        let mut world = World::new();
        let source = spawn_play_aura(
            &mut world,
            PlayerId::Player1,
            AuraEffect::GainAttack(1),
            AuraTarget::AllFriendlyMinions,
        );
        let target = spawn_play_minion(&mut world, PlayerId::Player1, 2, 3);
        assert_eq!(world.effective_attack(target), Some(Attack(3)));

        world.remove_aura(source);
        assert_eq!(world.effective_attack(target), Some(Attack(2)));
    }

    #[test]
    fn aura_source_leaves_play_invalidates_cache() {
        let mut world = World::new();
        let source = spawn_play_aura(
            &mut world,
            PlayerId::Player1,
            AuraEffect::GainAttack(1),
            AuraTarget::AllFriendlyMinions,
        );
        let target = spawn_play_minion(&mut world, PlayerId::Player1, 2, 3);
        assert_eq!(world.effective_attack(target), Some(Attack(3)));

        world.move_to_zone(source, Zone::Graveyard).unwrap();
        assert_eq!(world.effective_attack(target), Some(Attack(2)));
    }

    #[test]
    fn enemy_aura_applies_only_to_enemy_minions() {
        let mut world = World::new();
        spawn_play_aura(
            &mut world,
            PlayerId::Player2,
            AuraEffect::GainAttack(1),
            AuraTarget::AllEnemyMinions,
        );
        let p1_minion = spawn_play_minion(&mut world, PlayerId::Player1, 2, 3);
        let p2_minion = spawn_play_minion(&mut world, PlayerId::Player2, 2, 3);

        // P2's aura buffs P1's minion (P2's enemy)
        assert_eq!(world.effective_attack(p1_minion), Some(Attack(3)));
        // Does not affect P2's own minions
        assert_eq!(world.effective_attack(p2_minion), Some(Attack(2)));
    }

    #[test]
    fn gain_stats_buckets_both() {
        let mut world = World::new();
        spawn_play_aura(
            &mut world,
            PlayerId::Player1,
            AuraEffect::GainStats {
                attack: 1,
                health: 2,
            },
            AuraTarget::AllFriendlyMinions,
        );
        let target = spawn_play_minion(&mut world, PlayerId::Player1, 2, 3);

        assert_eq!(world.effective_attack(target), Some(Attack(3)));
        assert_eq!(world.effective_health(target), Some(Health(5)));
    }

    #[test]
    fn cost_reduction_aura_scoped_to_own_hand() {
        let mut world = World::new();
        let source = spawn_play_aura(
            &mut world,
            PlayerId::Player1,
            AuraEffect::ReduceSpellCost(1),
            AuraTarget::AllFriendlyMinions,
        );
        // A spell in P1's hand
        let spell = world.spawn();
        world.set_card_type(spell, CardType::Spell);
        world.set_cost(spell, Cost(2));
        world.set_player(spell, PlayerId::Player1);
        world.set_zone(spell, Zone::Hand);
        world
            .zones_mut()
            .insert(Zone::Hand, PlayerId::Player1, spell);
        // A minion in P1's hand
        let minion = world.spawn();
        world.set_card_type(minion, CardType::Minion);
        world.set_cost(minion, Cost(2));
        world.set_player(minion, PlayerId::Player1);
        world.set_zone(minion, Zone::Hand);
        world
            .zones_mut()
            .insert(Zone::Hand, PlayerId::Player1, minion);
        // A spell in P2's hand (must not be affected by P1's aura)
        let enemy_spell = world.spawn();
        world.set_card_type(enemy_spell, CardType::Spell);
        world.set_cost(enemy_spell, Cost(2));
        world.set_player(enemy_spell, PlayerId::Player2);
        world.set_zone(enemy_spell, Zone::Hand);
        world
            .zones_mut()
            .insert(Zone::Hand, PlayerId::Player2, enemy_spell);

        assert_eq!(world.effective_cost(spell), Some(Cost(1)));
        assert_eq!(world.effective_cost(minion), Some(Cost(2)));
        assert_eq!(world.effective_cost(enemy_spell), Some(Cost(2)));
        // The aura source itself is on the battlefield; its cost is not reduced
        assert_eq!(world.effective_cost(source), Some(Cost(0)));
    }

    #[test]
    fn minion_cost_floor_enforced() {
        let mut world = World::new();
        spawn_play_aura(
            &mut world,
            PlayerId::Player1,
            AuraEffect::ReduceMinionCost { amount: 2, min: 1 },
            AuraTarget::AllFriendlyMinions,
        );
        let minion = world.spawn();
        world.set_card_type(minion, CardType::Minion);
        world.set_cost(minion, Cost(1));
        world.set_player(minion, PlayerId::Player1);
        world.set_zone(minion, Zone::Hand);
        world
            .zones_mut()
            .insert(Zone::Hand, PlayerId::Player1, minion);

        // 1 cost minus 2 with a floor of 1 → 1 cost
        assert_eq!(world.effective_cost(minion), Some(Cost(1)));
    }

    #[test]
    fn player_change_rebuckets_aura() {
        let mut world = World::new();
        let source = spawn_play_aura(
            &mut world,
            PlayerId::Player1,
            AuraEffect::GainAttack(1),
            AuraTarget::OtherFriendlyMinions,
        );
        let target = spawn_play_minion(&mut world, PlayerId::Player1, 2, 3);
        assert_eq!(world.effective_attack(target), Some(Attack(3)));

        // Mind Control: source moves to P2 and is no longer a friendly minion of P1
        world.set_player(source, PlayerId::Player2);
        assert_eq!(world.effective_attack(target), Some(Attack(2)));
    }

    #[test]
    fn aura_source_despawn_invalidates_cache() {
        let mut world = World::new();
        let source = spawn_play_aura(
            &mut world,
            PlayerId::Player1,
            AuraEffect::GainAttack(1),
            AuraTarget::AllFriendlyMinions,
        );
        let target = spawn_play_minion(&mut world, PlayerId::Player1, 2, 3);
        assert_eq!(world.effective_attack(target), Some(Attack(3)));

        world.despawn(source);
        assert_eq!(world.effective_attack(target), Some(Attack(2)));
    }

    #[test]
    fn spawn_entity() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(world.is_alive(e));
        assert_eq!(e.index, 0);
        assert_eq!(e.generation, 0);
    }

    #[test]
    fn stale_handle_after_despawn() {
        let mut world = World::new();
        let e = world.spawn();
        world.despawn(e);
        assert!(!world.is_alive(e));
    }

    #[test]
    fn slot_reuse_bumps_generation() {
        let mut world = World::new();
        let e1 = world.spawn();
        assert_eq!(e1.index, 0);
        assert_eq!(e1.generation, 0);
        world.despawn(e1);

        // The old handle must be invalidated
        assert!(!world.is_alive(e1));

        // The slot is reused; the generation must differ
        let e2 = world.spawn();
        assert_eq!(e2.index, 0);
        assert_eq!(e2.generation, 1); // generation bump
        assert!(world.is_alive(e2));
        assert!(!world.is_alive(e1)); // old handle still invalid
    }

    #[test]
    fn component_set_and_get() {
        let mut world = World::new();
        let e = world.spawn();
        world.set_health(e, Health(30));
        world.set_attack(e, Attack(0));
        assert_eq!(world.health(e), Some(Health(30)));
        assert_eq!(world.attack(e), Some(Attack(0)));
    }

    #[test]
    fn component_missing_after_despawn() {
        let mut world = World::new();
        let e = world.spawn();
        world.set_health(e, Health(5));
        world.despawn(e);
        // Note: components are cleared, but the is_alive semantics must be checked
        // despawn does a generational check, and component removal also goes through generation
        // since the generation changed, the old handle cannot find the component
        assert_eq!(world.health(e), None);
    }

    #[test]
    fn move_to_zone_consistency() {
        let mut world = World::new();
        let e = world.spawn();
        world.set_player(e, PlayerId::Player1);
        world.set_zone(e, Zone::Hand);
        world.zones.insert(Zone::Hand, PlayerId::Player1, e);

        // Move to the battlefield
        world.move_to_zone(e, Zone::Play).unwrap();
        assert_eq!(world.zone(e), Some(Zone::Play));
        assert!(world.zones.is_empty(Zone::Hand, PlayerId::Player1));
        let play_entities: Vec<_> = world.zones.iter(Zone::Play, PlayerId::Player1).collect();
        assert_eq!(play_entities, vec![e]);
    }

    #[test]
    fn move_to_zone_maintains_order() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        for &(e, pid) in &[(e1, PlayerId::Player1), (e2, PlayerId::Player1)] {
            world.set_player(e, pid);
            world.set_zone(e, Zone::Hand);
            world.set_health(e, Health(0));
            world.zones.insert(Zone::Hand, pid, e);
        }
        // Move to the battlefield in order
        world.move_to_zone(e1, Zone::Play).unwrap();
        world.move_to_zone(e2, Zone::Play).unwrap();
        let play: Vec<_> = world.zones.iter(Zone::Play, PlayerId::Player1).collect();
        assert_eq!(play, vec![e1, e2]);
    }

    #[test]
    fn despawn_clears_zones() {
        let mut world = World::new();
        let e = world.spawn();
        world.set_player(e, PlayerId::Player1);
        world.set_zone(e, Zone::Play);
        world.zones.insert(Zone::Play, PlayerId::Player1, e);

        world.despawn(e);
        assert!(world.zones.is_empty(Zone::Play, PlayerId::Player1));
    }

    #[test]
    fn iter_components() {
        let mut world = World::new();
        let e1 = world.spawn();
        world.set_health(e1, Health(10));
        let e2 = world.spawn();
        world.set_health(e2, Health(20));
        let e3 = world.spawn();
        world.set_health(e3, Health(30));

        let mut healths: Vec<_> = world.iter_health().map(|(e, h)| (e.index, h.0)).collect();
        healths.sort_by_key(|(i, _)| *i);
        assert_eq!(healths, vec![(0, 10), (1, 20), (2, 30)]);
    }
}
