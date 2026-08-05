//! 卡牌定义模块 — 基本卡牌数据。
//!
//! 包含 CardDef 结构体、vanilla! 宏，以及所有怀旧系列卡牌定义。
//! 所有卡牌常量通过 `def` 模块统一 re-export，
//! 外部代码可通过 `crate::cards::def::*` 访问。

pub mod classic_druid;
pub mod classic_hunter;
pub mod classic_legendary;
pub mod classic_mage;
pub mod classic_neutral;
pub mod classic_paladin;
pub mod classic_priest;
pub mod classic_rogue;
pub mod classic_shaman;
pub mod classic_warlock;
pub mod classic_warrior;
pub mod def;
pub mod generated;
pub mod pool;
pub mod sets;

use crate::core::component::{
    Attack, AttacksUsed, Aura, CardId, Cost, Deathrattle, Durability, Health, Overload,
    OverloadTrigger, Poison, Stealth,
};
use crate::core::effect::{CardEffect, EffectTarget};
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::world::World;
use def::CardDef;

/// 在实体上应用特殊关键词组件（剧毒、潜行、过载等）。
///
/// 这些关键词不新增 `CardDef` 字段（避免大面积结构体改动），
/// 而是按卡牌 ID 在此集中映射。召唤随从（`trigger::resolve_summon`）
/// 和构建卡牌（`GameBuilder::spawn_minion`）时调用。
pub(crate) fn apply_card_keywords(world: &mut World, entity: Entity, card_def: &CardDef) {
    // 带过载的萨满卡牌（不模拟法力锁定，仅作为触发标记）
    if matches!(
        card_def.id,
        "SHAMAN_002"
            | "SHAMAN_006"
            | "SHAMAN_009"
            | "SHAMAN_015"
            | "SHAMAN_016"
            | "SHAMAN_017"
            | "SHAMAN_019"
    ) {
        world.set_overload(entity, Overload);
    }
    if card_def.id == "SHAMAN_021" {
        // 无羁元素 — 每当你使用一张带过载的牌时，获得 +1/+1
        world.set_overload_trigger(
            entity,
            OverloadTrigger(CardEffect::GainStats {
                attack: 1,
                health: 1,
                target: EffectTarget::Self_,
            }),
        );
    }
    if card_def.id == "ROGUE_022" {
        // 耐心的刺客 — 潜行 + 剧毒
        world.set_poison(entity, Poison);
        world.set_stealth(entity, Stealth);
    }
}

/// 清除随从的所有效果组件（变形前重置实体）。
///
/// 保留生命值/攻击力/区域等基础组件，由调用方重新套用目标卡牌的属性。
pub(crate) fn clear_minion_effects(world: &mut World, entity: Entity) {
    world.remove_battlecry(entity);
    world.remove_deathrattle(entity);
    world.remove_taunt(entity);
    world.remove_aura(entity);
    world.remove_secret(entity);
    world.remove_divine_shield(entity);
    world.remove_windfury(entity);
    world.remove_charge(entity);
    world.remove_spell_damage(entity);
    world.remove_cant_attack(entity);
    world.remove_end_turn_effect(entity);
    world.remove_spell_trigger(entity);
    world.remove_death_trigger(entity);
    world.remove_summon_trigger(entity);
    world.remove_choose_one_effect(entity);
    world.remove_combo_effect(entity);
    world.remove_attack_equals_health(entity);
    world.remove_temp_attack_debuff(entity);
    world.remove_poison(entity);
    world.remove_stealth(entity);
    world.remove_immune(entity);
    world.remove_freeze(entity);
    world.remove_overload(entity);
    world.remove_overload_trigger(entity);
}

/// 根据 `CardDef` 在指定玩家名下创建一个卡牌实体（不设置区域）。
///
/// 供 `GameBuilder` 与效果系统（如穆克拉的香蕉、随机卡池）共用，
/// 保证手牌/牌库中的卡牌实体携带完整的组件（战吼/亡语/关键词等）。
pub(crate) fn spawn_card_from_def(world: &mut World, player: PlayerId, card: &CardDef) -> Entity {
    let e = world.spawn();
    world.set_card_id(e, CardId(card.id));
    world.set_health(e, Health(card.health));
    world.set_attack(e, Attack(card.attack));
    world.set_cost(e, Cost(card.cost));
    world.set_card_type(e, card.card_type);
    world.set_player(e, player);
    world.set_attacks_used(e, AttacksUsed(0));
    // 设置武器耐久（如果是武器牌）
    if card.card_type == crate::core::component::CardType::Weapon && card.durability > 0 {
        world.set_durability(e, Durability(card.durability));
    }
    // 设置圣盾/风怒/冲锋/法伤
    if card.divine_shield {
        world.set_divine_shield(e, crate::core::component::DivineShield);
    }
    if card.windfury {
        world.set_windfury(e, crate::core::component::Windfury);
    }
    if card.charge {
        world.set_charge(e, crate::core::component::Charge);
    }
    if card.spell_damage != 0 {
        world.set_spell_damage(e, crate::core::component::SpellDamage(card.spell_damage));
    }
    // 设置光环（如果有）
    if let Some((aura_effect, aura_target)) = card.aura {
        world.set_aura(
            e,
            Aura {
                effect: aura_effect,
                target: aura_target,
            },
        );
    }
    // 设置战吼/亡语（已有字段）
    if let Some(bc) = card.battlecry {
        world.set_battlecry(e, crate::core::component::Battlecry(bc));
    }
    if let Some(dr) = card.deathrattle {
        world.set_deathrattle(e, Deathrattle(dr));
    }
    // 设置嘲讽
    if card.taunt {
        world.set_taunt(e, crate::core::component::Taunt);
    }
    // 设置不能攻击
    if card.cant_attack {
        world.set_cant_attack(e, crate::core::component::CantAttack);
    }
    // 设置回合结束效果
    if let Some(ete) = card.end_turn_effect {
        world.set_end_turn_effect(e, crate::core::component::EndTurnEffect(ete));
    }
    // 法术牌效果存储在 battlecry 组件中（打出时由引擎解析）
    if let Some(se) = card.spell_effect {
        world.set_battlecry(e, crate::core::component::Battlecry(se));
    }
    // 法术触发效果
    if let Some(st) = card.spell_trigger {
        world.set_spell_trigger(e, crate::core::component::SpellTrigger(st));
    }
    // 死亡触发效果
    if let Some(dt) = card.death_trigger {
        world.set_death_trigger(e, crate::core::component::DeathTrigger(dt));
    }
    // 召唤触发效果
    if let Some(st) = card.summon_trigger {
        world.set_summon_trigger(e, crate::core::component::SummonTrigger(st));
    }
    // 抉择效果
    if let Some(ce) = card.choose_one_effect {
        world.set_choose_one_effect(e, crate::core::component::ChooseOneEffect(ce));
    }
    // 连击效果
    if let Some(cb) = card.combo_effect {
        world.set_combo_effect(e, crate::core::component::ComboEffect(cb));
    }
    // 攻击力等于生命值
    if card.attack_equals_health {
        world.set_attack_equals_health(e, crate::core::component::AttackEqualsHealth);
    }
    // 特殊关键词（剧毒/潜行等）
    apply_card_keywords(world, e, card);
    e
}

#[cfg(test)]
mod generated_tests {
    use super::def::{CardDef, card_by_id};
    use crate::cards::generated;

    /// 生成的卡牌常量必须与手写常量逐字段一致（静态可表示的部分）。
    #[test]
    fn generated_cards_match_handwritten() {
        assert!(
            !generated::GENERATED_IDS.is_empty(),
            "generated registry must be non-empty"
        );
        for id in generated::GENERATED_IDS {
            let generated: CardDef = match find_generated(id) {
                Some(c) => c,
                None => panic!("generated const for {id} missing"),
            };
            let handwritten = card_by_id(id).unwrap_or_else(|| panic!("handwritten {id} missing"));
            // 静态字段完全一致；效果字段（战吼/亡语等）在生成代码中恒为 None，
            // 因此只对"静态可表示"的卡牌断言整体相等
            assert_eq!(generated.id, handwritten.id);
            assert_eq!(generated.name, handwritten.name);
            assert_eq!(generated.card_type, handwritten.card_type);
            assert_eq!(generated.cost, handwritten.cost);
            assert_eq!(generated.attack, handwritten.attack);
            assert_eq!(generated.health, handwritten.health);
            assert_eq!(generated.durability, handwritten.durability);
            assert_eq!(generated.taunt, handwritten.taunt);
            assert_eq!(generated.divine_shield, handwritten.divine_shield);
            assert_eq!(generated.windfury, handwritten.windfury);
            assert_eq!(generated.charge, handwritten.charge);
            assert_eq!(generated.spell_damage, handwritten.spell_damage);
        }
    }

    /// 纯静态卡牌（无任何效果字段）应整体相等。
    #[test]
    fn vanilla_generated_cards_fully_equal() {
        use crate::cards::def::BLOODFEN_RAPTOR;
        assert_eq!(
            find_generated("CLASSIC_001").unwrap(),
            BLOODFEN_RAPTOR,
            "vanilla card must be exactly equal"
        );
    }

    fn find_generated(id: &str) -> Option<CardDef> {
        // 通过注册表与命名规则定位生成的常量
        match id {
            "CLASSIC_001" => Some(generated::CLASSIC_001),
            "NEUTRAL_B02" => Some(generated::NEUTRAL_B02),
            "NEUTRAL_013" => Some(generated::NEUTRAL_013),
            "CLASSIC_014" => Some(generated::CLASSIC_014),
            _ => None,
        }
    }
}
