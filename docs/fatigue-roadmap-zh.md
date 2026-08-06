# 疲劳机制路线图 — 空牌库抽牌伤害

> 活跃路线图（尚未归档）。状态核实：2026-08-06（代码逐项核对）。本路线图清偿已登记的保真债 **F-A10**（`docs/fidelity-debt.md`）：引擎目前空牌库抽牌静默无效果，抽干牌库的对局会永远僵住，RL 环境被迫用 `max_steps=5000` 限步平局兜底。英文镜像：`fatigue-roadmap.md`。

---

## 1. 为什么做

| 痛点 | 现状 | 加疲劳后 |
| --- | --- | --- |
| 保真缺口（F-A10） | `draw_card_no_queue` 空牌库返回 `None`（`trigger.rs` — "fatigue in Phase 3+"）；无伤害、对局无进展 | 真实炉石规则：每次空牌库抽牌伤害递增 |
| 僵局 | 双方抽干牌库的对局永不结束；env 用 5000 步强制平局（`rl/env.rs`），battle 用 `max_turns` 兜底 | 对局自然以真实胜者结束；上限只剩兜底作用 |
| RL 正确性 | 抽牌多的卡组（术士生命分流、奥术智慧）的 agent 拿到的是永不终止、无信号的对局 | 疲劳是真实、可观测的压力；观测里的 deck_count=0 从此有意义 |
| SabberStone 对照 | 差分测试无法覆盖空牌库分支 | 疲劳语义与参考实现一致 |

## 2. 要实现的官方规则

炉石疲劳的精确语义：

1. **空牌库**上的**每次抽牌尝试**对抽牌英雄造成等于该玩家疲劳计数器的伤害，随后计数器 +1（第 1 次 = 1 点，第 2 次 = 2 点，……）。
2. 伤害**即时结算、按抽牌尝试逐次**发生。多张抽牌（奥术智慧：抽 2 张）空牌库时结算两次：1 点再 2 点。
3. 该伤害是**普通伤害**：英雄护甲吸收；致死则对局结束（受疲劳方判负）；防致死奥秘（寒冰屏障）对其生效。
4. 部分抽取：牌库剩 `k < N` 张时，先正常抽走 `k` 张，疲劳只覆盖余下 `N − k` 次尝试。
5. **不疲劳**的情形：种族扫描抽牌（感知恶魔抽到几张算几张，缺的不计伤害）和开局/换牌抽牌（牌库满，物理上不可达）。

## 3. 当前代码地图

| 部件 | 位置 | 说明 |
| --- | --- | --- |
| 抽牌咽喉点 | `src/engine/trigger.rs:848` `draw_card_no_queue` | 唯一的牌库顶抽牌；空牌库返回 `None` —— 注释预留的疲劳挂点（"fatigue in Phase 3+"） |
| 带队列的抽牌包装 | `src/engine/trigger.rs:857` `draw_card_with_reduction`（远视）、`:878` `draw_card` | 回合抽牌与卡牌效果都汇入此处 |
| 效果抽牌 | `src/engine/trigger.rs:79` `DrawCard`（逐张循环 —— 多张自然叠加）、`:621` `ChanceDraw`（纳特·帕格）、`:688` `DrawAndDamageByCost`（神圣愤怒）、亡语抽牌、术士生命分流英雄技能 | 全部经过咽喉点 |
| 种族扫描抽牌 | `src/engine/trigger.rs:886` `resolve_draw_by_race` | 直接扫描区域，不经咽喉点 —— 按规则 5 保持不动是正确的 |
| 开局/换牌抽牌（无队列） | `src/engine/rules.rs:1223`（换牌补抽）、`:1234`（第 4 张）、`src/core/state.rs:288/291` | 牌库必然非空；改签名不能把队列强加到这里 |
| 伤害管线 | `src/engine/rules.rs:907-985` `Event::DamageDealt` → 免疫 → 圣盾 → 护甲 → 生命 → 死亡判定 → `GameOver`（最高优先级） | 疲劳必须进这条管线，不能直接扣血 |
| 英雄致死奥秘 | `src/engine/secret.rs:122`（寒冰屏障） | 响应 `DamageDealt` 事件 —— 疲劳走管线后自动免费获得 |
| 玩家状态 | `src/core/player.rs:44` `Player` | 计数器归属地（已有 `armor`、`mana_crystals` 等） |
| RL 接口 | `src/rl/views.rs:83` `PlayerView`、`src/py_bind/views.rs` | 加 `fatigue` 字段；**168 维张量不动**（M0 基线约定 —— 与 rl-interface-roadmap D2-a 同理） |
| 对局兜底 | `src/rl/env.rs:223` `max_steps` 限步平局、`src/sim/battle.rs:444` `max_turns` | 保留，仅作兜底 |

## 4. 设计决策

### D1 — 计数器位置与语义
`Player.fatigue: u32`，**从 1 起**：计数器本身即下一次空牌库抽牌将造成的伤害，随后 `counter += 1`。（否决：0 起 + 结算时 +1 —— 1 起制可直接读出伤害值，与炉石描述一致。）随 `Serialize/Deserialize` 派生免费获得序列化往返。

### D2 — 挂点：抽牌咽喉点
疲劳放在 `draw_card_no_queue` —— 所有抽牌路径（回合抽牌、卡牌效果、英雄技能、亡语、纳特·帕格、神圣愤怒）都汇聚于此，一处改动全覆盖，未来的卡也不会漏掉疲劳。签名变更：

- `draw_card_no_queue(state, player)` → `draw_card_no_queue(state, queue: &mut EventQueue, player)`；空牌库时：推入 `Event::DamageDealt { source: hero, target: hero, amount: counter }`，随后 `counter += 1`，返回 `None`（**不发** `CardDrawn` —— "每当你抽一张牌"的触发不得被误触发）。
- 三处开局/换牌调用点（规则 5 —— 牌库必然满）改用轻量无队列助手 `draw_top_card_no_queue`，带非空 `debug_assert!` 不变量，开局热路径保持无队列形态。

### D3 — 疲劳伤害走统一管线
`source = 抽牌玩家自己的英雄实体`（英雄无剧毒，剧毒判定自然无效）。由此免费获得的正确行为：护甲吸收；致死走既有死亡判定 → `GameOver`；寒冰屏障响应 `DamageDealt` 事件。与 SabberStone 一致。

### D4 — 种族扫描抽牌保持不疲劳语义
`resolve_draw_by_race` 不动，加注释说明原因（规则 5）。"抽 N 张某种族随从"的效果找到 0 张就抽 0 张、0 伤害 —— 感知恶魔在 HS 中即如此。

### D5 — RL 暴露：只走结构化视图
`PlayerView.fatigue: u32`（rl 与 py_bind 两处视图），让 Python 能读到计数器。**168 维张量形状不改**（与 rl-interface-roadmap D2-a 同理：改张量破坏 M0 基线约定；RL 侧从结构化视图构造特征）。

### D6 — 兜底保留
`max_steps` / `max_turns` 保留为保险（理论上只有护甲/治疗循环能熬过疲劳）。F-A10 账本条目更新为"已解决 —— 疲劳已实现；上限降级为兜底"。

## 5. 里程碑

### M1 — 核心规则（引擎） ✅
- [ ] `Player.fatigue: u32`（1 起）加入 `src/core/player.rs`
- [ ] `draw_card_no_queue` 增加 `&mut EventQueue` 参数；空牌库 → `DamageDealt`（source = 英雄，amount = 计数器），计数器 +1，返回 `None`
- [ ] 三处开局/换牌调用点改用 `draw_top_card_no_queue` 助手（带非空 `debug_assert!`）
- [ ] 测试（新 `tests/fatigue.rs` 或并入 `tests/gameplay.rs`）：
  - 首次空牌库抽牌 1 点、第二次 2 点
  - 空牌库多张抽牌（抽 2）造成 1 + 2 = 3 点
  - 回合抽牌疲劳（DrawStep 空牌库）
  - 护甲吸收疲劳伤害（英雄技能叠甲 / 盾牌格挡）
  - 疲劳致死 → `GameOver`，胜者正确
  - 牌库非空时抽牌行为不变（无伤害、牌入手）
  - 开局/换牌抽牌不受影响（不变量成立）

**验收**：`cargo test` 全绿（基线 405 项）；以上每条行为都有具名测试钉住。

### M2 — 覆盖与交互
- [ ] 逐条审计所有抽牌路径确认都经咽喉点（回合抽牌、`DrawCard`、`ChanceDraw`、`DrawAndDamageByCost`、亡语、生命分流、远视、战斗怒火、照明弹）；任何未覆盖的直接 `zones().iter(Zone::Deck)` 移动需给出决策（多半是 D4 式"不疲劳"或重构）
- [ ] **F-A10 场景正式钉住**：双方空牌库 → 对局远在 `max_steps` 之前以真实胜者结束（用疲劳致死测试替代旧的"限步平局"预期）
- [ ] 寒冰屏障：疲劳致死被阻止，对局继续
- [ ] 确定性：疲劳不使用 RNG —— 同 seed 重放逐位一致（既有确定性测试必须原样通过）
- [ ] SabberStone 差分用例覆盖空牌库对局（`tests/differential.rs`，若对照矩阵覆盖到疲劳分支）
- [ ] `cargo bench` 抽查：疲劳是冷路径（每局至多约 30 次），热路径无回退

**验收**：全套测试通过；`GameEnv` 与 `battle` 两条路径下，双方空牌库对局都以真实胜者结束。

### M3 — RL 接口与文档
- [ ] `src/rl/views.rs` + `src/py_bind/views.rs` 加 `PlayerView.fatigue`，绑定冒烟测试（工具侧）
- [ ] `docs/fidelity-debt.md` F-A10 条目标记已解决（保留"上限降级为兜底"的说明）
- [ ] `trigger.rs` 的 "fatigue in Phase 3+" 注释替换为真实语义
- [ ] 本路线图对（中英）归档到 `docs/finished/`，去掉"活跃"标注

**验收**：Python 绑定能读到计数器并在疲劳对局中正确取值；两份文档语言一致。

## 6. 超出范围

- **手牌上限烧牌**（满手抽牌弃掉新牌）—— 另一个既有缺口；若 `fidelity-debt.md` 机制清单未登记，去那里登记，不在这里。
- 168 维观测张量加疲劳计数器（D5）。
- 新卡牌或超出咽喉点改动的引擎重构。
