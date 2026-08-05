//! World — ECS 容器，管理所有实体和组件。
//!
//! World 是实体生命周期的唯一入口：
//! - `spawn()` 创建实体（分配槽位）
//! - `despawn()` 销毁实体（释放槽位，清除所有组件和区域引用）
//! - `move_to_zone()` 原子性地将实体从一个区域移动到另一个区域
//! - 组件通过生成的 accessor 方法访问
//!
//! 所有实体访问都经过 generation 检查，防止悬垂引用。

use crate::core::component::{
    Armor, Attack, AttackEqualsHealth, AttacksUsed, Aura, Battlecry, CantAttack, CardId, CardType,
    Charge, ChooseOneEffect, ComboEffect, Cost, DeathTrigger, Deathrattle, DivineShield,
    Durability, EndTurnEffect, Freeze, Health, HeroPowerDef, HeroPowerUsed, Poison, Secret,
    SpellDamage, SpellTrigger, Stealth, SummonTrigger, Taunt, TempAttackDebuff, Windfury,
};
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::sparse_set::SparseSet;
use crate::core::zone::{Zone, ZoneError, Zones};

/// 区域移动错误 — move_to_zone 的可能失败模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// 实体已销毁
    EntityGone,
    /// 缺少 PlayerId 组件
    MissingPlayer,
    /// 缺少当前 Zone 组件
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

/// 生成组件访问方法的宏。
///
/// 为每种组件类型生成 get/set/remove/iter 四个方法。
macro_rules! component_accessors {
    ($field:ident, $t:ty, $get:ident, $set:ident, $remove:ident, $iter:ident) => {
        #[doc = concat!("获取实体的 `", stringify!($t), "` 组件。")]
        #[must_use]
        pub fn $get(&self, entity: Entity) -> Option<$t> {
            self.$field.get(entity)
        }

        #[doc = concat!("设置实体的 `", stringify!($t), "` 组件。")]
        pub fn $set(&mut self, entity: Entity, value: impl Into<$t>) {
            self.$field.insert(entity, value.into());
        }

        #[doc = concat!("移除实体的 `", stringify!($t), "` 组件。")]
        pub fn $remove(&mut self, entity: Entity) -> Option<$t> {
            self.$field.remove(entity)
        }

        #[doc = concat!("遍历所有拥有 `", stringify!($t), "` 组件的实体。")]
        pub fn $iter(&self) -> impl Iterator<Item = (Entity, &$t)> {
            self.$field.iter()
        }
    };
}

/// ECS World — 所有实体和组件的容器。
///
/// # 内部结构
///
/// - `generations`: 每个槽位的代际版本号（despawn 时递增）
/// - `free_list`: 可复用的空闲槽位（FIFO）
/// - 10 个组件稀疏集 + Zones 表
#[derive(Debug, Clone)]
pub struct World {
    /// 每个槽位的代际版本号，用于检测过期 Entity handle
    generations: Vec<u32>,
    /// 可复用的空闲槽位索引
    free_list: Vec<u32>,
    /// Health 组件存储
    health: SparseSet<Health>,
    /// Attack 组件存储
    attack: SparseSet<Attack>,
    /// Cost 组件存储
    cost: SparseSet<Cost>,
    /// CardType 组件存储
    card_type: SparseSet<CardType>,
    /// Zone 组件存储（实体的当前位置）
    zone_comp: SparseSet<Zone>,
    /// PlayerId 组件存储
    player: SparseSet<PlayerId>,
    /// AttacksUsed 组件存储
    attacks_used: SparseSet<AttacksUsed>,
    /// Battlecry 组件存储
    battlecry: SparseSet<Battlecry>,
    /// Deathrattle 组件存储
    deathrattle: SparseSet<Deathrattle>,
    /// Taunt 组件存储
    taunt: SparseSet<Taunt>,
    /// Durability 组件存储（武器耐久）
    durability: SparseSet<Durability>,
    /// Armor 组件存储（英雄护甲）
    armor: SparseSet<Armor>,
    /// HeroPowerDef 组件存储（英雄技能定义）
    hero_power: SparseSet<HeroPowerDef>,
    /// HeroPowerUsed 组件存储（本回合是否已使用技能）
    hero_power_used: SparseSet<HeroPowerUsed>,
    /// Aura 组件存储（光环效果）
    aura: SparseSet<Aura>,
    /// Secret 组件存储（奥秘）
    secret: SparseSet<Secret>,
    /// DivineShield 组件存储（圣盾）
    divine_shield: SparseSet<DivineShield>,
    /// Windfury 组件存储（风怒）
    windfury: SparseSet<Windfury>,
    /// Charge 组件存储（冲锋）
    charge: SparseSet<Charge>,
    /// SpellDamage 组件存储（法术伤害加成）
    spell_damage: SparseSet<SpellDamage>,
    /// Freeze 组件存储（冻结）
    freeze: SparseSet<Freeze>,
    /// CantAttack 组件存储（不能攻击）
    cant_attack: SparseSet<CantAttack>,
    /// EndTurnEffect 组件存储（回合结束效果）
    end_turn_effect: SparseSet<EndTurnEffect>,
    /// SpellTrigger 组件存储（法术触发效果）
    spell_trigger: SparseSet<SpellTrigger>,
    /// DeathTrigger 组件存储（随从死亡触发效果）
    death_trigger: SparseSet<DeathTrigger>,
    /// SummonTrigger 组件存储（随从召唤触发效果）
    summon_trigger: SparseSet<SummonTrigger>,
    /// ChooseOneEffect 组件存储（抉择效果）
    choose_one_effect: SparseSet<ChooseOneEffect>,
    /// ComboEffect 组件存储（连击效果）
    combo_effect: SparseSet<ComboEffect>,
    /// AttackEqualsHealth 组件存储（光耀之子）
    attack_equals_health: SparseSet<AttackEqualsHealth>,
    /// TempAttackDebuff 组件存储（临时攻击减益）
    temp_attack_debuff: SparseSet<TempAttackDebuff>,
    /// CardId 组件存储（原始卡牌定义 ID）
    card_id: SparseSet<CardId>,
    /// Poison 组件存储（剧毒）
    poison: SparseSet<Poison>,
    /// Stealth 组件存储（潜行）
    stealth: SparseSet<Stealth>,
    /// 区域表 — 每个 Zone 的有序实体列表
    zones: Zones,
}

impl World {
    /// 创建一个空的世界。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_list: Vec::new(),
            health: SparseSet::new(),
            attack: SparseSet::new(),
            cost: SparseSet::new(),
            card_type: SparseSet::new(),
            zone_comp: SparseSet::new(),
            player: SparseSet::new(),
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
            end_turn_effect: SparseSet::new(),
            spell_trigger: SparseSet::new(),
            death_trigger: SparseSet::new(),
            summon_trigger: SparseSet::new(),
            choose_one_effect: SparseSet::new(),
            combo_effect: SparseSet::new(),
            attack_equals_health: SparseSet::new(),
            temp_attack_debuff: SparseSet::new(),
            card_id: SparseSet::new(),
            poison: SparseSet::new(),
            stealth: SparseSet::new(),
            zones: Zones::new(),
        }
    }

    /// 生成一个新的实体并返回其句柄。
    ///
    /// 优先复用空闲槽位，否则扩展数组。
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

    /// 检查实体是否还存活（generation 匹配）。
    #[must_use]
    pub fn is_alive(&self, entity: Entity) -> bool {
        let idx = entity.index as usize;
        idx < self.generations.len() && self.generations[idx] == entity.generation
    }

    /// 销毁实体：清除所有组件，从所有区域移除，递增 generation，归还槽位。
    ///
    /// Phase 1 中 despawn 仅用于清理（测试等场景）。
    /// 游戏内死亡使用 `move_to_zone(entity, Zone::Graveyard)` 而非 despawn。
    pub fn despawn(&mut self, entity: Entity) {
        if !self.is_alive(entity) {
            return;
        }
        let idx = entity.index as usize;
        // 从所有区域移除
        self.zones.remove_from_all(entity);
        // 清除所有组件
        self.health.remove(entity);
        self.attack.remove(entity);
        self.cost.remove(entity);
        self.card_type.remove(entity);
        self.zone_comp.remove(entity);
        self.player.remove(entity);
        self.attacks_used.remove(entity);
        self.battlecry.remove(entity);
        self.deathrattle.remove(entity);
        self.taunt.remove(entity);
        self.durability.remove(entity);
        self.armor.remove(entity);
        self.hero_power.remove(entity);
        self.hero_power_used.remove(entity);
        self.aura.remove(entity);
        self.secret.remove(entity);
        self.divine_shield.remove(entity);
        self.windfury.remove(entity);
        self.charge.remove(entity);
        self.spell_damage.remove(entity);
        self.freeze.remove(entity);
        self.cant_attack.remove(entity);
        self.end_turn_effect.remove(entity);
        self.spell_trigger.remove(entity);
        self.death_trigger.remove(entity);
        self.summon_trigger.remove(entity);
        self.choose_one_effect.remove(entity);
        self.combo_effect.remove(entity);
        self.attack_equals_health.remove(entity);
        self.temp_attack_debuff.remove(entity);
        self.card_id.remove(entity);
        self.poison.remove(entity);
        self.stealth.remove(entity);
        // 提升 generation
        self.generations[idx] = self.generations[idx].wrapping_add(1);
        // 归还槽位
        self.free_list.push(entity.index);
    }

    /// 将实体从一个区域移动到另一个区域。
    ///
    /// 这是区域转移的**唯一入口**，确保 Zone 组件和 Zones 表保持同步。
    ///
    /// # 错误
    ///
    /// - `MoveError::EntityGone` — 实体已销毁
    /// - `MoveError::MissingPlayer` — 缺少 PlayerId 组件，无法判断所属玩家
    /// - `MoveError::MissingZone` — 缺少当前 Zone 组件（状态不一致）
    pub fn move_to_zone(&mut self, entity: Entity, target: Zone) -> Result<(), MoveError> {
        if !self.is_alive(entity) {
            return Err(MoveError::EntityGone);
        }
        let player = self.player(entity).ok_or(MoveError::MissingPlayer)?;
        let current = self.zone(entity).ok_or(MoveError::MissingZone)?;

        // 从旧区域移除
        self.zones.remove(current, player, entity);
        // 插入新区域
        self.zones.insert(target, player, entity);
        // 更新 Zone 组件
        self.set_zone(entity, target);

        Ok(())
    }

    /// 获取 Zones 表的只读引用。
    #[must_use]
    pub fn zones(&self) -> &Zones {
        &self.zones
    }

    /// 获取 Zones 表的可变引用（用于测试/GameBuilder 直接操作区域）。
    ///
    /// ⚠️ 直接操作 Zones 表需要同时更新 Zone 组件，否则状态不一致。
    /// 优先使用 `move_to_zone`。
    pub fn zones_mut(&mut self) -> &mut Zones {
        &mut self.zones
    }

    // 为每种组件类型生成 accessor 方法
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
    component_accessors!(zone_comp, Zone, zone, set_zone, remove_zone, iter_zone);
    component_accessors!(
        player,
        PlayerId,
        player,
        set_player,
        remove_player,
        iter_player
    );
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
    component_accessors!(aura, Aura, aura, set_aura, remove_aura, iter_aura);
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
        end_turn_effect,
        EndTurnEffect,
        end_turn_effect,
        set_end_turn_effect,
        remove_end_turn_effect,
        iter_end_turn_effect
    );
    component_accessors!(
        spell_trigger,
        SpellTrigger,
        spell_trigger,
        set_spell_trigger,
        remove_spell_trigger,
        iter_spell_trigger
    );
    component_accessors!(
        death_trigger,
        DeathTrigger,
        death_trigger,
        set_death_trigger,
        remove_death_trigger,
        iter_death_trigger
    );
    component_accessors!(
        summon_trigger,
        SummonTrigger,
        summon_trigger,
        set_summon_trigger,
        remove_summon_trigger,
        iter_summon_trigger
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
    component_accessors!(
        temp_attack_debuff,
        TempAttackDebuff,
        temp_attack_debuff,
        set_temp_attack_debuff,
        remove_temp_attack_debuff,
        iter_temp_attack_debuff
    );
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
    component_accessors!(
        stealth,
        Stealth,
        stealth,
        set_stealth,
        remove_stealth,
        iter_stealth
    );

    /// 获取实体每回合可攻击的最大次数。
    #[must_use]
    pub fn max_attacks(&self, entity: Entity) -> u8 {
        if self.windfury(entity).is_some() {
            2
        } else {
            1
        }
    }

    /// 获取场上友方法术伤害加成总和。
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

    /// 获取实体的有效攻击力（基础攻击力 + 所有光环加成）。
    ///
    /// 遍历场上所有带 `Aura` 组件的存活实体，检查当前实体是否在光环范围内，
    /// 累积叠加所有匹配光环的攻击力加成。
    #[must_use]
    pub fn effective_attack(&self, entity: Entity) -> Option<Attack> {
        let base = self.attack(entity)?;
        let player = self.player(entity)?;

        let mut bonus = 0i32;
        for (aura_entity, aura) in self.iter_aura() {
            // 跳过不存活的光环源
            if !self.is_alive(aura_entity) {
                continue;
            }
            // 光环源必须在战场上
            if self.zone(aura_entity) != Some(crate::core::zone::Zone::Play) {
                continue;
            }
            let aura_player = match self.player(aura_entity) {
                Some(p) => p,
                None => continue,
            };

            if aura_applies_to(aura, aura_entity, aura_player, entity, player, self) {
                bonus += aura_attack_bonus(aura.effect);
            }
        }

        Some(Attack(base.0 + bonus))
    }

    /// 获取实体的有效生命值（基础生命值 + 所有光环加成）。
    ///
    /// 遍历场上所有带 `Aura` 组件的存活实体，累积叠加匹配光环的生命值加成。
    #[must_use]
    pub fn effective_health(&self, entity: Entity) -> Option<Health> {
        let base = self.health(entity)?;
        let player = self.player(entity)?;

        let mut bonus = 0i32;
        for (aura_entity, aura) in self.iter_aura() {
            if !self.is_alive(aura_entity) {
                continue;
            }
            if self.zone(aura_entity) != Some(crate::core::zone::Zone::Play) {
                continue;
            }
            let aura_player = match self.player(aura_entity) {
                Some(p) => p,
                None => continue,
            };

            if aura_applies_to(aura, aura_entity, aura_player, entity, player, self) {
                bonus += aura_health_bonus(aura.effect);
            }
        }

        Some(Health(base.0 + bonus))
    }

    /// 获取实体的有效法力消耗（基础费用 - 费用减免光环）。
    ///
    /// 遍历场上所有存活的光环源，累加匹配的费用减免：
    /// - `ReduceSpellCost` 作用于手牌中的友方法术（巫师学徒）
    /// - `ReduceMinionCost` 作用于手牌中的友方随从，且不低于费用下限
    ///   （召唤传送门 — 至少 1 费）
    #[must_use]
    pub fn effective_cost(&self, entity: Entity) -> Option<Cost> {
        use crate::core::component::AuraEffect;

        let base = self.cost(entity)?;
        let player = self.player(entity)?;
        let card_type = self.card_type(entity)?;
        let in_hand = self.zone(entity) == Some(crate::core::zone::Zone::Hand);

        let mut reduction = 0i32;
        let mut min_cost = 0i32;
        for (aura_entity, aura) in self.iter_aura() {
            if !self.is_alive(aura_entity) {
                continue;
            }
            if self.zone(aura_entity) != Some(crate::core::zone::Zone::Play) {
                continue;
            }
            // 费用光环只影响光环拥有者自己的手牌
            if self.player(aura_entity) != Some(player) {
                continue;
            }
            match aura.effect {
                AuraEffect::ReduceSpellCost(amount) if in_hand && card_type == CardType::Spell => {
                    reduction += amount;
                }
                AuraEffect::ReduceMinionCost { amount, min }
                    if in_hand && card_type == CardType::Minion =>
                {
                    reduction += amount;
                    min_cost = min_cost.max(min);
                }
                _ => {}
            }
        }
        Some(Cost((base.0 - reduction).max(min_cost)))
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 光环辅助函数
// ============================================================

/// 检查光环效果是否作用于目标实体。
fn aura_applies_to(
    aura: &Aura,
    aura_source: Entity,
    aura_player: PlayerId,
    target: Entity,
    target_player: PlayerId,
    world: &World,
) -> bool {
    use crate::core::component::{AuraTarget, CardType};

    // 目标必须是存活的随从
    if world.card_type(target) != Some(CardType::Minion) {
        return false;
    }
    if !world.is_alive(target) {
        return false;
    }

    match aura.target {
        AuraTarget::AllFriendlyMinions => target_player == aura_player,
        AuraTarget::OtherFriendlyMinions => target_player == aura_player && target != aura_source,
        AuraTarget::AdjacentMinions => {
            if target_player != aura_player || target == aura_source {
                return false;
            }
            is_adjacent(aura_source, target, aura_player, world)
        }
        AuraTarget::AllEnemyMinions => target_player != aura_player,
    }
}

/// 检查两个实体在战场上是否相邻。
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
            // 相邻 = 位置差为 1
            (s as isize - t as isize).unsigned_abs() == 1
        }
        _ => false,
    }
}

/// 返回光环效果的攻击力加成。
const fn aura_attack_bonus(effect: crate::core::component::AuraEffect) -> i32 {
    use crate::core::component::AuraEffect;
    match effect {
        AuraEffect::GainStats { attack, .. } => attack,
        AuraEffect::GainAttack(a) => a,
        AuraEffect::GainHealth(_) => 0,
        AuraEffect::ReduceSpellCost(_) => 0,
        AuraEffect::ReduceMinionCost { .. } => 0,
    }
}

/// 返回光环效果的生命值加成。
const fn aura_health_bonus(effect: crate::core::component::AuraEffect) -> i32 {
    use crate::core::component::AuraEffect;
    match effect {
        AuraEffect::GainStats { health, .. } => health,
        AuraEffect::GainAttack(_) => 0,
        AuraEffect::GainHealth(h) => h,
        AuraEffect::ReduceSpellCost(_) => 0,
        AuraEffect::ReduceMinionCost { .. } => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        // 旧句柄应失效
        assert!(!world.is_alive(e1));

        // 复用槽位，generation 应该不同
        let e2 = world.spawn();
        assert_eq!(e2.index, 0);
        assert_eq!(e2.generation, 1); // generation bump
        assert!(world.is_alive(e2));
        assert!(!world.is_alive(e1)); // 旧句柄仍然失效
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
        // 注意：组件被清除但需要检查 is_alive 语义
        // despawn 后 generational check，但组件 remove 也是通过 generation
        // 由于 generation 已变，旧句柄查不到组件
        assert_eq!(world.health(e), None);
    }

    #[test]
    fn move_to_zone_consistency() {
        let mut world = World::new();
        let e = world.spawn();
        world.set_player(e, PlayerId::Player1);
        world.set_zone(e, Zone::Hand);
        world.zones.insert(Zone::Hand, PlayerId::Player1, e);

        // 移到战场
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
        // 按顺序移到战场
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
