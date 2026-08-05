//! build.rs — 从官方格式卡牌 JSON 生成 `CardDef` 常量（roadmap D2）。
//!
//! 输入：`cards/classic_cards.json`（HearthstoneJSON 风格的子集）
//! 输出：`OUT_DIR/cards_generated.rs` — 每张卡一个 `pub const <ID>: CardDef`，
//!       以及 `pub const GENERATED_IDS: &[&str]` 注册表。
//!
//! 说明：官方 JSON 只包含静态属性与关键词；战吼/亡语/光环等脚本化效果
//! 仍需手写（`CardDef` 中对应的 `Option` 字段在生成代码中为 `None`）。
//! 用完整官方数据库替换 `cards/classic_cards.json`（保持字段名一致），
//! 即可批量生成全部"静态可表示"的卡牌，消除手工录入静态属性的工作。
//!
//! 生成的卡牌与手写常量在 `cards/mod.rs` 的测试中逐字段比对验证。

use std::env;
use std::fs;
use std::path::Path;

/// 官方 JSON 卡牌的子集字段。
#[derive(serde::Deserialize)]
struct JsonCard {
    id: String,
    name: String,
    #[serde(default)]
    cost: i32,
    #[serde(default)]
    attack: i32,
    #[serde(default)]
    health: i32,
    #[serde(default)]
    durability: i32,
    #[serde(default)]
    #[serde(rename = "type")]
    card_type: String,
    #[serde(default)]
    mechanics: Vec<String>,
}

/// 将卡牌 ID 转为合法的 Rust 常量名（ID 本身即合法标识符时原样使用）。
fn const_name(id: &str) -> String {
    let sanitized: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() || sanitized.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("CARD_{sanitized}")
    } else {
        sanitized
    }
}

fn main() {
    println!("cargo:rerun-if-changed=cards/classic_cards.json");
    let json_path = Path::new("cards/classic_cards.json");
    let raw = fs::read_to_string(json_path).expect("read cards/classic_cards.json");
    let cards: Vec<JsonCard> = serde_json::from_str(&raw).expect("parse cards/classic_cards.json");

    let mut code =
        String::from("// 由 build.rs 自动生成 — 勿手改（数据源：cards/classic_cards.json）。\n\n");
    let mut ids = Vec::new();
    for card in &cards {
        let name = const_name(&card.id);
        let card_type = match card.card_type.as_str() {
            "SPELL" => "CardType::Spell",
            "WEAPON" => "CardType::Weapon",
            _ => "CardType::Minion",
        };
        let taunt = card.mechanics.iter().any(|m| m == "TAUNT");
        let divine_shield = card.mechanics.iter().any(|m| m == "DIVINE_SHIELD");
        let windfury = card.mechanics.iter().any(|m| m == "WINDFURY");
        let charge = card.mechanics.iter().any(|m| m == "CHARGE");
        let spell_damage = card.mechanics.iter().filter(|m| *m == "SPELLPOWER").count() as i32;

        code.push_str(&format!(
            "/// 生成的卡牌常量（来自官方 JSON）：{} — {}\n",
            card.id, card.name
        ));
        code.push_str(&format!("pub const {name}: CardDef = CardDef {{\n"));
        code.push_str(&format!("    id: \"{}\",\n", card.id));
        code.push_str(&format!("    name: \"{}\",\n", card.name));
        code.push_str(&format!("    card_type: {card_type},\n"));
        code.push_str(&format!("    cost: {},\n", card.cost));
        code.push_str(&format!("    attack: {},\n", card.attack));
        code.push_str(&format!("    health: {},\n", card.health));
        code.push_str(&format!("    durability: {},\n", card.durability));
        code.push_str("    battlecry: None,\n    deathrattle: None,\n");
        code.push_str(&format!("    taunt: {taunt},\n"));
        code.push_str("    hero_power: None,\n    aura: None,\n    secret: None,\n");
        code.push_str(&format!("    divine_shield: {divine_shield},\n"));
        code.push_str(&format!("    windfury: {windfury},\n"));
        code.push_str(&format!("    charge: {charge},\n"));
        code.push_str(&format!("    spell_damage: {spell_damage},\n"));
        code.push_str("    cant_attack: false,\n    end_turn_effect: None,\n");
        code.push_str("    start_turn_effect: None,\n    spell_effect: None,\n");
        code.push_str("    spell_trigger: None,\n    death_trigger: None,\n");
        code.push_str("    summon_trigger: None,\n    choose_one_effect: None,\n");
        code.push_str("    combo_effect: None,\n    attack_equals_health: false,\n};\n\n");
        ids.push(card.id.clone());
    }

    code.push_str("/// 生成卡牌的 ID 注册表。\n");
    code.push_str("pub const GENERATED_IDS: &[&str] = &[\n");
    for id in &ids {
        code.push_str(&format!("    \"{id}\",\n"));
    }
    code.push_str("];\n");

    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join("cards_generated.rs");
    fs::write(&dest, code).expect("write generated cards");
}
