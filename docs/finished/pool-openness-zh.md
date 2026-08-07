# 开放池卡 —— 常设注册表

> **开放池卡**（pool-open）的权威注册表：结算时会把某张牌搬进某个区域、而这张牌并非
> 从 `cards/pool.rs` 控制的池里抽出来的卡。英文对照：`pool-openness.md`。

## 契约

一张卡是**开放池卡**，当且仅当它的结算会把某张牌搬进某个区域，而这张牌并非从
`cards/pool.rs` 控制的池里抽出来的。目前唯一属于此类的是 4 张**读对手真实牌**（手牌
或牌库）或**复制被施放的法术**的卡：

| 卡牌 | 职业 | 费用 | 为什么开放池 | 落地时间 |
| --- | --- | --- | --- | --- |
| 心灵视界（`PRIEST_024`） | 牧师 | 1 | 复制**敌方手牌**随机一张 | 2026-08-07，M2 |
| 思维窃取（`PRIEST_025`） | 牧师 | 3 | 复制**敌方牌库** 2 张 | 2026-08-07，M2 |
| 心灵游戏（`PRIEST_026`） | 牧师 | 4 | 召唤**敌方牌库**随机随从的复制 | 2026-08-07，M2 |
| 游学者周卓（`LEGENDARY_024`） | 中立传说 | 2 | 复制**被施放的法术**给另一名玩家 | 2026-08-07，M3 |

经典卡池今天处于**封闭**状态：`ALL_CARDS` 全是经典卡，对手手里的东西本来就在池内；
其余所有随机产牌都从 `ALL_CARDS` 的过滤子集或固定衍生物池里抽样。这个注册表的存在，
是为了将来支持第二个系列时封闭性仍然可审计 —— 上面这 4 张是唯一能把牌跨池搬运的卡。

**强制检查**（在代码里，`cargo test`）：

1. 读区域效果变体（`CopyRandomEnemyHandCard`、`CopyRandomEnemyDeckCards`、
   `SummonRandomEnemyDeckMinion`、`CopyCastSpellToOtherPlayerHand`）**只允许**出现在
   `sets::POOL_OPEN_CARDS` 里的卡上 —— 由 `pool_open_effects_require_registry`
   钉住。
2. `pool_open_registry_is_well_formed`：注册表里每个 ID 都能被 `card_by_id` 解析、
   不是衍生物（`t` 结尾）、无重复。
3. `cards::apply_card_keywords` 里游学者周卓的钩子会对「卡已登记」做 debug_assert。

## 维护约定

- **新增开放池卡**：把 ID 登记进 `sets::POOL_OPEN_CARDS`，在 `src/cards/` 里对应卡
  的文档块写 `(pool-open: …)` 注记，并给本表加一行 —— 三件事必须在同一次改动里完成。
- **不是保真债**：开放池卡是*忠实*实现。注释里**不许**出现 "simplified" 字样 ——
  Python 欠债提取器（`hearthstone_os/decks.py::_load_debt_ids`）认的是 "simplified"，
  不能把它们捞进去（见 `fidelity-debt.md` 维护约定）。
- **RL 卡池**：`full_pool(include_pool_open=True)` 收进它们；把默认值翻成 `False`
  即可闭池，引擎一行不用动。
