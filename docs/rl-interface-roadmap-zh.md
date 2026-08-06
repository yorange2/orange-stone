# RL 接口路线图 — 面向 orange-reinforcement 的外部接口工作

> 本路线图记录 orange-stone 面向 RL 训练/对拍的**外部接口工作**（绑定层 API、批量、卡牌视图与保真），由 orange-reinforcement 对接项目驱动（RL 侧文档：`orange-reinforcement/docs/roadmap.md`）。引擎内部的架构与保真里程碑（A~G、F）见 [architecture-roadmap.md](architecture-roadmap.md)；本文档是它的 RL 接口补充——对接项目提出 G2~G8 差距后逐一补齐，并承载 M4 批量绑定与 M5 保真债清偿。
>
> 现状核对时间：2026-08-05（M1 起点，代码均已实地核实）。

---

## 1. 为什么 RL 侧选择 orange-stone

orange-reinforcement 上一轮"接真实引擎"的尝试是 RosettaStone C++ 绑定（`rosetta/`），有三个硬伤，orange-stone 全部解决（详见 RL 侧文档 §1）：

| RosettaStone 的痛点 | orange-stone 的对应能力 |
| --- | --- |
| `Game` 不可 clone（拷贝/移动构造 `= delete`），整回合搜索做不了 | CoW GameState 克隆廉价，搜索/回滚天然可恢复 |
| 全局静态 RNG，并行采样被迫多进程 | 每局独立 `GameRng`，`sim/batch.rs` 用 rayon 线程级并行且逐局可复现 |
| AGPL-3.0（许可证传染） | MIT |

## 2. 对接前的 RL 接口现状（M0 基线）

| 已有 | 位置 | 说明 |
| --- | --- | --- |
| RL 环境 | `src/rl/env.rs` | `GameEnv`：单智能体 vs 内置 bot；合法动作全枚举；`max_steps=5000` 防死循环 |
| 观察编码 | `src/rl/obs.rs` | 固定 **168 维**（英雄/法力/牌堆计数/手牌 10 槽×6/双方场上 7 槽×7），已归一化。**只含基础关键词**（嘲讽/圣盾/风怒/冲锋/潜行），没有战吼/亡语/触发/卡面文本 |
| 奖励 | `src/rl/reward.rs` | 可配置（胜负平 + 每步密化项）；默认稀疏 win+1/loss−1/draw 0 |
| Python 绑定 | `src/py_bind/mod.rs` | `GameEnv(seed, perspective=0, deck_size=30)` → `reset` / `observation` / `legal_actions`（`[(index, 字符串描述)]`）/ `step`（→ `(obs, reward, done, winner)`） |
| 批量模拟 | `src/sim/batch.rs` | `BatchSimulator`：rayon 并行 bot-vs-bot，per-game RNG 逐局可复现；**只有 bot 驱动，没有 RL 批量 step** |
| 内置 bot | `src/sim/battle.rs` | `BotType::Greedy` / `Smart`，仅此两个 |

## 3. 绑定层差距（RL 对接提出，G2~G8）

| # | 差距 | 现状 | 影响 |
| --- | --- | --- | --- |
| G2 | 无自定义卡组 | 卡组从全卡池随机生成（`deck_size` 张、单副内不重复） | 对拍/训练需要固定池 ×2 镜像（简化炉石的口径） |
| G3 | 无结构化观察/动作 | 只有 168 维向量 + `(index, 字符串描述)` | RL 特征工程（251 维 v6）需要结构化字段：动作类型/手牌下标/攻击者/目标/交易结果 |
| G4 | 双方不可同时外部控制 | `GameEnv` 固定"智能体 vs 内置 bot"，EndTurn 后 bot 自动代打 | arena（bot vs bot）、将来的 self-play 需要无 bot 模式 |
| G5 | clone 未暴露 | Rust 侧 CoW 便宜，但 py_bind 没有 clone | 整回合 beam 搜索无法在 Python 侧恢复（rosetta 就是死在这） |
| G6 | 起手规则固定 | 双方各 3 张，无后手 4 张 + 幸运币 | 与简化炉石口径不一致，先手优势失真 |
| G7 | 奖励口径不同 | 稀疏胜负 +1/−1 | 简化炉石是"输按对手剩血给 0~−1"，训练曲线不具可比性 |
| G8 | 无 RL 批量 step | `batch.rs` 只支持 bot 驱动 | 训练并行采样要自己拿 multiprocessing 拼 |

（G1 绑定安装、G9 卡池不对齐、G10 规则语义差异的适配属 RL 侧，见 `orange-reinforcement/docs/roadmap.md` §2.3。）

## 4. 里程碑

### M1 — 接口补齐（约 1~2 周）✅ 已完成（2026-08-05）

按 G2~G8 逐个补，每个改动带 Rust 单测 + 绑定层测试：

- [x] **自定义卡组**（G2）：`EnvConfig`/py_bind 支持显式卡组列表（固定池 ×2 镜像），随机模式保留（`GameEnv(seed, deck=[...])`，未知 ID 抛 ValueError）
- [x] **结构化 Action/Observation 视图**（G3）：仿 rosetta 绑定（`EntityView/PlayerView/Observation` 形状）在 PyO3 里导出结构化字段；字符串描述继续保留给 play.py 用（`structured_observation()` / `structured_legal_actions()`）
- [x] **双方可控模式**（G4）：`BotType::None`，EndTurn 后当前玩家变为可被外部 step；arena 和训练共用（`GameEnv(seed, bot="none")`）
- [x] **暴露 clone()**（G5）：py_bind 加 `clone()`（Rust 侧 `GameEnv: Clone` 已 derive，只需透传）
- [x] **起手可配**（G6）：hand_size、后手硬币开关（后手 4 张 + 幸运币，硬币不占牌库）
- [x] **奖励口径参数化**（G7）：把简化炉石的 `final_reward`（赢 +1 / 平 0 / 输 0~−1 按剩血）作为可选 reward config 导出（`terminal_reward="sparse"|"health_scaled"`）
- [x] ~~（可选）`obs.rs` 增补卡面文本维度~~：**不做**——G3 的结构化视图已带 card_id/name/全部关键词，RL 侧特征 v7 按决策 D2-a 从结构化视图构建，无需改动 168 维张量（改了反而破坏 M0 基线口径）

**验收**：以上每项在 `tests/` 有 Rust 测试；Python 侧有对应 smoke 测试。**已达成**：`cargo test` 321 全过；RL 侧 `tools/orange_stone_m1_smoke.py` 六个小节全过；M0 冒烟回归（确定性 8 seed）通过。

**M1 实测结果**（2026-08-05，逐项 PR 合并）：6 个 PR **#64**（G2 卡组）、**#65**（G3 视图）、**#66**（G4 无 bot）、**#67**（G5 clone）、**#68**（G6 起手）、**#69**（G7 奖励）；RL 侧对应 6 个冒烟 PR（`orange_stone_m1_smoke.py` 随每个 PR 追加小节）。过程中修的两个预埋 bug：小卡组发牌漏牌（按原牌库长度取索引）、固定卡组开局与 seed 无关（洗牌/起手改为 runner RNG 驱动）。

### M4 — 批量与性能（引擎部分，约 1 周）✅ 已完成（2026-08-06）

RL 侧并行训练的先决条件是**绑定层释放 GIL**：

- [x] **GIL 释放**（**#71**）：热方法包 `py.allow_threads`——引擎纯 Rust + 逐局 RNG 线程安全；此前 GIL 把线程完全串行
- [x] **批量观测/动作张量化**：`orange_stone.BatchEnv`（一次调用驱动 N 局、结构化观测给**当前行动方视角**——批量训练免掉 RL 侧双实例锁步、`reset_one` 单独重开）、`battle_batch`（rayon 批量，直接暴露 `sim/batch.rs`）
- [x] 吞吐：`battle_batch` ~4,200 局/s（≈9.2× rosetta 460，目标 ≥10× 未完全达成但同量级）

**验收**：RL 侧 `test_batched.py`——确定性策略下 BatchedEnv 与单局 Env 12 局 winner 完全一致 + `battle_batch` 单局/批量一致；基准表入 RL 侧 README。

**注意**：引擎批量已达 ~4,200 局/s，但训练侧吞吐仍受 Python 特征/前向的 GIL 串行限制（RL 侧事项；要再上量需把特征与前向批量下沉或 GPU 批量推理）。

### M5 — 卡池扩展与保真（引擎部分，与里程碑 F 同步）✅ 已完成（2026-08-06）

- [x] **卡牌视图/卡池补充**（**#72**/**#73**/**#74**/**#75**）：潜行卡（`CardDef.stealth`）、扰咒机制（`CardDef.elusive` + 法术目标枚举/结算排除）、卡面文本视图字段（card_type + effect 量级）、`all_card_ids()`
- [x] **与里程碑 F 同步**：F1~F5 内部完成（`tests/differential.rs`）；**外部 SabberStone 对照跑通**（**#75**：dotnet 驱动镜像 attack-trade 场景，两个模拟器结果一致，见 `docs/differential_sabberstone.md`）
- [x] **保真债清偿**（F4/F5 持续审计项）✅ **全部清偿（2026-08-06）**：67 处简化卡标记记录在 `docs/finished/fidelity-debt.md`（审计账本，已归档且为空），执行计划 `docs/finished/fidelity-debt-roadmap.md`（8 个按依赖排序的 wave：W0 接线 13 张 → W1 种族 11 → W2 触发 8 → W3 谓词 9 → W4 费用武器 8 → W5 目标结构 7 → W6 特殊机制 8 → W7 收尾 3）已全部落地（PR #79–#86），每张卡"实现 + F5 差分验证"后离开账本（`tests/differential.rs` 共 72 个 `w0_*`–`w7_*` 场景）；F4/F5 持续审计机制保留

**过程中补的账本外修正**：
- **#77 结构性发现**：过期注释、Worgen Infiltrator 补潜行、3 处卡 ID 冲突、10 张卡补入 ALL_CARDS、7 个重复条目——ALL_CARDS 现为 **413 唯一条目**
- **顺手牵羊（Pilfer）**：OtherClass 卡池原过滤是"任意非潜行者"（把中立卡也算进去），已收窄为另外 8 个职业的职业卡

**RL 侧联动**：保真债清偿后训练卡池扩到全经典构筑规模 **391 张**；`decks.py::_load_debt_ids` 的卡池执行 bug（简化注释错记到上一张卡 ID）在 RL 侧 PR #31 修复。细节见 `orange-reinforcement/docs/roadmap.md` §3 M5。

## 5. 遗留与风险（引擎侧）

| 风险 | 对策 |
| --- | --- |
| 个别卡可能仍有简化（F4 持续审计中） | 训练卡池只用**已实现且通过 differential 的卡**；新增简化卡按归档账本的维护约定登记 |
| 性能敏感路径回退（`sim/`、`rl/`） | 改动需跑 `cargo bench`，别让对拍/训练吞吐回退 |

## 6. 相关文档

- RL 侧对接路线图：`orange-reinforcement/docs/roadmap.md`（M0/M2/M3/M6、决策点 D1~D5、风险）
- 架构路线图：`docs/architecture-roadmap.md`（里程碑 G 是 F 的前置，F4/F5 是保真债的载体；本文档 M5 是其 RL 侧落地记录）
- 保真债账本与执行计划（已归档）：`docs/finished/fidelity-debt.md`、`docs/finished/fidelity-debt-roadmap.md`
- 外部对照：`docs/differential_sabberstone.md`（SabberStone 差分验证协议）
