//! 卡牌效果定义 — 编译时常量的 CardEffect 和 EffectTarget。
//!
//! Phase 2 支持的卡牌效果：伤害、抽牌、召唤、buff。
//! 效果作为 `Copy` 枚举常量存储在 `CardDef` 和 `Battlecry`/`Deathrattle` 组件中。

/// 效果目标选择器。
///
/// 执行效果时，引擎根据此枚举选择目标实体。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectTarget {
    /// 随机敌方角色（英雄或随从）
    AnyEnemy,
    /// 随机敌方随从
    AnyEnemyMinion,
    /// 所有敌方随从
    AllEnemyMinions,
    /// 所有敌方角色（敌方英雄 + 所有敌方随从）
    AllEnemies,
    /// 敌方英雄
    EnemyHero,
    /// 自身（buff 类效果）
    Self_,
    /// 所有友方随从
    AllFriendlyMinions,
    /// 所有随从（不分敌我）
    AllMinions,
    /// 所有角色（英雄+随从，不分敌我）
    AllCharacters,
    /// 友方英雄
    FriendlyHero,
    /// 受伤的敌方随从
    DamagedEnemyMinion,
    /// 随机友方随从
    FriendlyMinion,
    /// 随机敌方嘲讽随从
    TauntEnemyMinion,
}

/// 卡牌效果 — 触发时执行的动作。
///
/// 实现 `Copy` 以作为组件存储。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardEffect {
    /// 造成 N 点伤害（DealDamage）
    DealDamage {
        /// 伤害数值
        amount: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 抽 N 张牌
    DrawCard {
        /// 抽牌数量
        count: u32,
    },
    /// 召唤一个随从
    SummonMinion {
        /// 要召唤的卡牌 ID
        card_id: &'static str,
    },
    /// 获得 +N/+M (buff 自身或友方)
    GainStats {
        /// 攻击力增量
        attack: i32,
        /// 生命值增量
        health: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 装备武器
    EquipWeapon {
        /// 要装备的武器卡牌 ID
        card_id: &'static str,
    },
    /// 获得护甲
    GainArmor {
        /// 护甲值
        amount: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 将一个随从移回手牌
    ReturnToHand {
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 增加随从的法力消耗（冰冻陷阱效果）
    IncreaseCost {
        /// 法力消耗增量
        amount: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 将一个随从移回手牌并使其法力消耗增加（冰冻陷阱完整效果）
    ReturnToHandAndIncreaseCost {
        /// 法力消耗增量
        amount: i32,
    },
    /// 消灭随从（暗言术：灭、刺杀）
    DestroyMinion {
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 沉默随从 — 移除所有效果组件
    SilenceMinion {
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 设置攻击力为固定值
    SetAttack {
        /// 目标攻击力
        attack: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 恢复生命值
    RestoreHealth {
        /// 恢复量
        amount: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 冻结角色
    FreezeCharacter {
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 获得空法力水晶
    GainManaCrystal {
        /// 获得数量
        count: i32,
    },
    /// 摧毁敌方武器
    DestroyWeapon,
    /// 给英雄增加临时攻击力和可选护甲（本回合有效，回合结束时清除攻击力加成）
    GainHeroAttack {
        /// 攻击力加成
        attack: i32,
        /// 护甲加成（0 表示不加护甲）
        armor: i32,
    },
    /// 对目标造成等于英雄攻击力的伤害
    DealHeroAttackDamage {
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 将一个随从的生命值恢复到满
    FullHeal {
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 给一个随从增加风怒
    GrantWindfury {
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 给一个随从增加冲锋和可选的攻击力
    GrantCharge {
        /// 目标选择方式
        target: EffectTarget,
        /// 额外攻击力（0 表示不加攻击力）
        attack_bonus: i32,
    },
    /// 双倍一个随从的攻击力
    DoubleAttack {
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 双倍一个随从的生命值
    DoubleHealth {
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 给友方武器增加攻击力和耐久度
    BuffWeapon {
        /// 攻击力增量
        attack: i32,
        /// 耐久度增量
        durability: i32,
    },
    /// 随机丢弃一张手牌
    DiscardRandomCard,
    /// 对目标造成等于友方英雄护甲值的伤害
    DealArmorDamage {
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 摧毁敌方武器并抽等于其耐久度的牌数
    DestroyWeaponAndDraw,
    /// 返回所有随从到各自拥有者手牌
    ReturnAllToHand,
    /// 将随从的攻击力设为等于其当前生命值
    SetAttackToHealth {
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 消灭所有随从，除随机一个之外
    DestroyAllExceptOne,
    /// 消灭一个随从并为己方英雄恢复生命值
    DestroyAndHeal {
        /// 目标选择方式
        target: EffectTarget,
        /// 恢复量
        heal: i32,
    },
    /// 消灭一个友方随从，对其攻击力数值造成AOE伤害
    DestroyAndAOE {
        /// 目标：所有敌方随从 / 所有敌方角色
        target: EffectTarget,
    },
    /// 对两个随机敌方随从造成伤害
    DealDamageToTwo {
        /// 伤害数值
        amount: i32,
    },
    /// 造成伤害 + 抽牌
    DealDamageAndDraw {
        /// 伤害数值
        damage: i32,
        /// 目标选择方式
        target: EffectTarget,
        /// 抽牌数量
        draw: u32,
    },
    /// 对一个随从造成伤害并获得攻击力
    DamageAndGainAttack {
        /// 伤害数值
        damage: i32,
        /// 攻击力加成
        attack_bonus: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 消灭一个友方随从并获得其攻击力和生命值
    DestroyAdjacent {
        /// 是否获得属性值
        gain_stats: bool,
    },
    /// 摧毁一颗法力水晶
    DestroyManaCrystal,
    /// 给对手手牌中添加卡牌
    GiveCardsToOpponent {
        /// 要添加的卡牌数量
        count: u32,
    },
    /// 复活一个本回合死亡的友方随从，生命值为1
    ResurrectMinion,
    /// 复制一个随机友方随从的攻击力和生命值
    CopyMinionStats,
    /// 给敌方随从-2攻击力（本回合有效）
    TempDebuff {
        /// 攻击力减少量
        attack_reduction: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 将受到的反伤反射给攻击者（奥秘效果）
    ReflectDamage,
    /// 对目标造成伤害，并在连击时将施放者移回手牌（头部爆裂）
    DealDamageAndReturnToHand {
        /// 伤害数值
        amount: i32,
        /// 目标选择方式
        target: EffectTarget,
    },
    /// 将一个友方随从移回手牌并使其费用减少（暗影步）
    ReturnFriendlyToHandAndReduceCost {
        /// 费用减少量
        amount: i32,
    },
    /// 对目标相邻的随从造成等于其攻击力的伤害（背叛）
    AdjacentDamage,
    /// 摧毁己方武器并对所有敌人造成等于其攻击力的伤害（剑刃乱舞）
    DestroyWeaponAndDealAttackToEnemies,
    /// 使一个友方随从获得潜行（伪装大师）
    GrantStealth,
    /// 召唤多个随从（毒蛇陷阱的三条蛇、塞纳留斯的双树人）
    SummonMultipleMinions {
        /// 要召唤的卡牌 ID
        card_id: &'static str,
        /// 召唤数量
        count: u32,
    },
    /// 对刚被打出的敌方随从造成伤害（狙击 — 由 secret.rs 处理，需要事件上下文）
    DamagePlayedMinion {
        /// 伤害数值
        amount: i32,
    },
    /// 将攻击重定向到另一个随机角色（误导 — 由 secret.rs 处理）
    RedirectAttackToRandomCharacter,
    /// 召唤一个随从作为攻击的新目标（崇高牺牲 — 由 secret.rs 处理）
    SummonAndRedirectAttack {
        /// 要召唤的防御者卡牌 ID
        card_id: &'static str,
    },
    /// 召唤 1/3 法术扭曲者并重定向法术伤害（法术扭曲者 — 由 secret.rs 处理）
    SummonSpellbender,
    /// 你的下一个奥秘费用为 (0)（肯瑞托法师）
    NextSecretCostsZero,
    /// 抽一张牌并使其费用减少（视界术）
    DrawCardAndReduceCost {
        /// 费用减少量
        amount: i32,
    },
}
