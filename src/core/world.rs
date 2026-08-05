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
    Durability, EndTurnEffect, Freeze, Health, HeroPowerDef, HeroPowerUsed, Immune, Overload,
    OverloadTrigger, Poison, Secret, SpellDamage, SpellTrigger, Stealth, SummonTrigger, Taunt,
    TempAttackDebuff, Windfury,
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
    /// 活跃光环源索引（见 [`AuraIndex`]）— 由所有相关变异方法增量维护。
    aura_index: AuraIndex,
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
    player_comp: SparseSet<PlayerId>,
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
    /// Immune 组件存储（免疫）
    immune: SparseSet<Immune>,
    /// Overload 组件存储（过载标记）
    overload: SparseSet<Overload>,
    /// OverloadTrigger 组件存储（过载触发效果）
    overload_trigger: SparseSet<OverloadTrigger>,
    /// 区域表 — 每个 Zone 的有序实体列表
    zones: Zones,
}

/// 光环源索引 — 将存活的战场光环源按（拥有者, 效果种类）分桶。
///
/// 光环适用性取决于源的三个易变属性（存活、在战场、拥有者）以及效果种类。
/// 每次查询都扫描全部 `Aura` 组件是 O(实体数 × 光环数)；此索引将每个查询
/// 限制到可能影响它的分桶。
///
/// 索引由 `World` 的相关变异方法增量维护（`set_aura`/`remove_aura`/
/// `set_zone`/`set_player`/`despawn`），查询路径是纯只读，无锁。
///
/// 注意：索引只信任 `zone` 组件（与查询语义一致），不信任 `Zones` 表，
/// 因此 `zones_mut()` 直接操作区域表不会造成索引与查询不一致。
#[derive(Debug, Clone, PartialEq)]
struct AuraIndex {
    /// 影响攻击力的光环源（按拥有者分桶）
    attack: [Vec<(Entity, Aura)>; 2],
    /// 影响生命值的光环源（按拥有者分桶）
    health: [Vec<(Entity, Aura)>; 2],
    /// 减少费用的光环源（按拥有者分桶）
    cost: [Vec<(Entity, Aura)>; 2],
}

impl AuraIndex {
    /// 创建一个空索引。
    #[must_use]
    const fn new() -> Self {
        Self {
            attack: [const { Vec::new() }, const { Vec::new() }],
            health: [const { Vec::new() }, const { Vec::new() }],
            cost: [const { Vec::new() }, const { Vec::new() }],
        }
    }

    /// 从所有分桶中移除实体。
    fn remove_entity(&mut self, entity: Entity) {
        for player in 0..PlayerId::COUNT {
            self.attack[player].retain(|(e, _)| *e != entity);
            self.health[player].retain(|(e, _)| *e != entity);
            self.cost[player].retain(|(e, _)| *e != entity);
        }
    }

    /// 按效果种类将实体加入对应分桶。
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
            AuraEffect::GainAttack(_) => self.attack[oi].push((entity, aura)),
            AuraEffect::GainHealth(_) => self.health[oi].push((entity, aura)),
            AuraEffect::ReduceSpellCost(_) | AuraEffect::ReduceMinionCost { .. } => {
                self.cost[oi].push((entity, aura))
            }
        }
    }
}

impl World {
    /// 创建一个空的世界。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            aura_index: AuraIndex::new(),
            generations: Vec::new(),
            free_list: Vec::new(),
            health: SparseSet::new(),
            attack: SparseSet::new(),
            cost: SparseSet::new(),
            card_type: SparseSet::new(),
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
            immune: SparseSet::new(),
            overload: SparseSet::new(),
            overload_trigger: SparseSet::new(),
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
        self.immune.remove(entity);
        self.overload.remove(entity);
        self.overload_trigger.remove(entity);
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
    ///
    /// 光环索引只信任 `Zone` 组件（与查询语义一致），不信任此表，
    /// 因此直接操作区域表不会造成索引与查询不一致。
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
    /// 获取实体的 `Zone` 组件。
    #[must_use]
    pub fn zone(&self, entity: Entity) -> Option<Zone> {
        self.zone_comp.get(entity)
    }

    /// 设置实体的 `Zone` 组件（可能改变光环适用性，增量维护索引）。
    pub fn set_zone(&mut self, entity: Entity, value: impl Into<Zone>) {
        let value = value.into();
        // 光环源进出战场会改变其"活跃"状态，需要同步索引
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

    /// 移除实体的 `Zone` 组件（使光环索引失去该实体的活跃状态信息）。
    pub fn remove_zone(&mut self, entity: Entity) -> Option<Zone> {
        let removed = self.zone_comp.remove(entity);
        if removed.is_some() && self.aura.contains(entity) {
            self.aura_index.remove_entity(entity);
        }
        removed
    }

    /// 遍历所有拥有 `Zone` 组件的实体。
    pub fn iter_zone(&self) -> impl Iterator<Item = (Entity, &Zone)> {
        self.zone_comp.iter()
    }

    /// 获取实体的 `PlayerId` 组件。
    #[must_use]
    pub fn player(&self, entity: Entity) -> Option<PlayerId> {
        self.player_comp.get(entity)
    }

    /// 设置实体的 `PlayerId` 组件（拥有者变化改变光环分桶，增量维护索引）。
    pub fn set_player(&mut self, entity: Entity, value: impl Into<PlayerId>) {
        let value = value.into();
        // 活跃光环源换边：先从原分桶移除，再按新拥有者重新加入
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

    /// 移除实体的 `PlayerId` 组件（使光环索引失去该实体的活跃状态信息）。
    pub fn remove_player(&mut self, entity: Entity) -> Option<PlayerId> {
        let removed = self.player_comp.remove(entity);
        if removed.is_some() && self.aura.contains(entity) && self.zone(entity) == Some(Zone::Play)
        {
            self.aura_index.remove_entity(entity);
        }
        removed
    }

    /// 遍历所有拥有 `PlayerId` 组件的实体。
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
    /// 获取实体的 `Aura` 组件。
    #[must_use]
    pub fn aura(&self, entity: Entity) -> Option<Aura> {
        self.aura.get(entity)
    }

    /// 设置实体的 `Aura` 组件（光环源集合可能变化，增量维护索引）。
    pub fn set_aura(&mut self, entity: Entity, value: impl Into<Aura>) {
        let value = value.into();
        // 活跃光环源更新效果：先从索引移除，再按新值重新加入
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

    /// 移除实体的 `Aura` 组件（从索引中移除该光环源）。
    pub fn remove_aura(&mut self, entity: Entity) -> Option<Aura> {
        let removed = self.aura.remove(entity);
        if removed.is_some() {
            self.aura_index.remove_entity(entity);
        }
        removed
    }

    /// 遍历所有拥有 `Aura` 组件的实体。
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
    component_accessors!(
        overload_trigger,
        OverloadTrigger,
        overload_trigger,
        set_overload_trigger,
        remove_overload_trigger,
        iter_overload_trigger
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
    /// 通过光环源索引只扫描可能影响攻击力的光环（己方与敌方的攻击桶），
    /// 而不是遍历全部 `Aura` 组件。
    #[must_use]
    pub fn effective_attack(&self, entity: Entity) -> Option<Attack> {
        let base = self.attack(entity)?;
        let player = self.player(entity)?;

        let mut bonus = 0i32;
        // 己方光环影响己方随从，敌方光环通过 AllEnemyMinions 影响己方随从
        for owner in [player, player.opponent()] {
            for (source, aura) in &self.aura_index.attack[owner.index()] {
                if aura_applies_to(aura, *source, owner, entity, player, self) {
                    bonus += aura_attack_bonus(aura.effect);
                }
            }
        }

        Some(Attack(base.0 + bonus))
    }

    /// 获取实体的有效生命值（基础生命值 + 所有光环加成）。
    ///
    /// 通过光环源索引只扫描可能影响生命值的光环（己方与敌方的生命桶）。
    #[must_use]
    pub fn effective_health(&self, entity: Entity) -> Option<Health> {
        let base = self.health(entity)?;
        let player = self.player(entity)?;

        let mut bonus = 0i32;
        for owner in [player, player.opponent()] {
            for (source, aura) in &self.aura_index.health[owner.index()] {
                if aura_applies_to(aura, *source, owner, entity, player, self) {
                    bonus += aura_health_bonus(aura.effect);
                }
            }
        }

        Some(Health(base.0 + bonus))
    }

    /// 获取实体的有效法力消耗（基础费用 - 费用减免光环）。
    ///
    /// 通过光环源索引只扫描己方的费用桶：
    /// - `ReduceSpellCost` 作用于手牌中的友方法术（巫师学徒）
    /// - `ReduceMinionCost` 作用于手牌中的友方随从，且不低于费用下限
    ///   （召唤传送门 — 至少 1 费）
    #[must_use]
    pub fn effective_cost(&self, entity: Entity) -> Option<Cost> {
        use crate::core::component::AuraEffect;

        let base = self.cost(entity)?;
        let player = self.player(entity)?;
        let card_type = self.card_type(entity)?;
        let in_hand = self.zone(entity) == Some(Zone::Hand);

        // 费用光环只影响光环拥有者自己的手牌
        let mut reduction = 0i32;
        let mut min_cost = 0i32;
        for (_, aura) in &self.aura_index.cost[player.index()] {
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
    use crate::core::component::{AuraEffect, AuraTarget, CardType};

    /// 生成一个位于战场的随从。
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

    /// 生成一个位于战场的光环源。
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

    /// 从权威数据（光环组件集 + 存活/战场/拥有者检查）暴力重建索引，
    /// 用于验证增量维护与"重建"语义完全一致。
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
                AuraEffect::GainAttack(_) => idx.attack[oi].push((source, *aura)),
                AuraEffect::GainHealth(_) => idx.health[oi].push((source, *aura)),
                AuraEffect::ReduceSpellCost(_) | AuraEffect::ReduceMinionCost { .. } => {
                    idx.cost[oi].push((source, *aura))
                }
            }
        }
        idx
    }

    /// 断言增量维护的索引与暴力重建结果一致（忽略桶内顺序）。
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

        // 手牌中的光环卡（有 Aura 组件但不活跃）
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

        // 召唤到战场 → 变为活跃
        world.move_to_zone(hand_aura, Zone::Play).unwrap();
        assert_index_matches(&world);

        // 活跃时更新光环值
        world.set_aura(
            hand_aura,
            Aura {
                effect: AuraEffect::GainAttack(2),
                target: AuraTarget::AllFriendlyMinions,
            },
        );
        assert_index_matches(&world);

        // 精神控制：换边
        world.set_player(hand_aura, PlayerId::Player2);
        assert_index_matches(&world);

        // 光环值覆盖为 GainStats（进两个桶）
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

        // 移除光环
        world.remove_aura(hand_aura);
        assert_index_matches(&world);

        // 重新挂光环，然后击杀（离开战场）
        world.set_aura(hand_aura, attack_aura);
        assert_index_matches(&world);
        world.move_to_zone(hand_aura, Zone::Graveyard).unwrap();
        assert_index_matches(&world);

        // 费用光环源 + 销毁
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
        // 生命值不受攻击光环影响
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

        // P2 的光环攻击 P1 的随从（P2 的敌人）
        assert_eq!(world.effective_attack(p1_minion), Some(Attack(3)));
        // 不影响 P2 自己的随从
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
        // P1 手牌中的法术
        let spell = world.spawn();
        world.set_card_type(spell, CardType::Spell);
        world.set_cost(spell, Cost(2));
        world.set_player(spell, PlayerId::Player1);
        world.set_zone(spell, Zone::Hand);
        world
            .zones_mut()
            .insert(Zone::Hand, PlayerId::Player1, spell);
        // P1 手牌中的随从
        let minion = world.spawn();
        world.set_card_type(minion, CardType::Minion);
        world.set_cost(minion, Cost(2));
        world.set_player(minion, PlayerId::Player1);
        world.set_zone(minion, Zone::Hand);
        world
            .zones_mut()
            .insert(Zone::Hand, PlayerId::Player1, minion);
        // P2 手牌中的法术（不应被 P1 的光环影响）
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
        // 光环源本身在战场，费用不减免
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

        // 1 费减 2 且下限 1 → 1 费
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

        // 精神控制：source 转移到 P2，不再是 P1 的友方随从
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
