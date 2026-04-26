# PRD: Privy SVM — Solana 可编程隐私虚拟机

> Product Requirements Document | Version 1.0 | 2026-04-26

---

## 1. 产品概述

### 1.1 一句话定义

Privy SVM 是 Solana 上的**可编程隐私虚拟机**——开发者用一行 `#[private]` 注解即可让链上程序拥有私有状态、私有输入和私有输出，链上可验证但状态不可见。

### 1.2 核心价值主张

> "Write Rust. Add `#[private]`. Get ZK privacy. Zero learning curve."

Solana 的开发者不需要学 Noir、不需要学 Circom、不需要理解 Groth16 和 Plonk 的数学。他们只需要在 Rust 变量上标注 `#[private]`，Privy SVM 自动完成一切。

### 1.3 解决的根问题

**Solana 的核心结构性缺陷：所有程序状态 100% 公开。**

| 因缺乏隐私而**完全不可行**的应用 | 经济损失规模 |
|---|---|
| 密封竞价 / 拍卖 | DeFi 用户年均被 MEV 提取 $50M+ |
| 暗池 / 大宗交易 | 鲸鱼每次交易被 front-run，滑点损失 2-5% |
| 链上策略游戏 | GameFi 完全放弃大额博彩市场 |
| DAO 隐私薪酬 | 几乎 100% 团队使用中心化工具解决 |
| 企业国库 / 财务 | 无大型企业在 Solana 上管钱 |
| 隐私信用评分 | 无链上隐私借贷生态 |

### 1.4 市场时机

- **Tornado Cash 事件（2022）** 后的监管空白 → 市场急需**合规隐私**方案
- **2025 年 ZK 证明生成速度**已从分钟级降到毫秒级，浏览器端可行
- **Solana Firedancer 客户端**（Jump）2025 上线后网络性能翻倍，需要隐私层配合
- **2026 年机构 DeFi 化**趋势明确，但合规隐私是唯一瓶颈

---

## 2. 目标用户

### 2.1 核心用户画像

| 角色 | 需求 | 使用方式 |
|---|---|---|
| **Solana 开发者** | 给我的程序加隐私 | 在 Rust 代码里加 `#[private]` 注解 |
| **DeFi 协议** | 保护用户隐私 | 集成 Privy SVM 作为隐私执行层 |
| **GameFi 开发者** | 隐藏游戏状态 | 调用 `#[private]` 游戏引擎 |
| **DAO** | 隐私投票/薪酬 | 部署隐私投票程序 |
| **机构** | 合规隐私 | 选择性披露给审计方 |
| **终端用户** | 用隐私应用 | 在浏览器中生成 ZK 证明，然后上链 |

### 2.2 用户故事

```
作为 Solana 开发者，我希望：
  - 用 Rust 写隐私逻辑，不需要学 Noir/Circom
  - 一行注解就能把变量变隐私
  - 隐私程序和公开程序可以互相调用
  - ZK 电路的编译是自动的，我不需要管
  - 有一个 npm/rust crate 可以直接用在我的前端

作为终端用户，我希望：
  - 使用隐私应用时，我的资产/持仓/行为不被公开
  - 生成 ZK 证明只需要 1 秒，不阻塞体验
  - 我能验证应用是真正隐私的（开源可审计）
  - 我可以随时撤销我之前发布的证明
  - 我的隐私不会因为我用的协议被黑而泄露

作为 DeFi 协议方，我希望：
  - 合规地验证用户身份（KYC/AML）而不存储用户数据
  - 给用户提供隐私交易选项作为竞争优势
  - 隐私层的开销(费用/延迟)足够低，不影响正常使用
```

---

## 3. 功能需求

### 3.1 核心功能

#### P0 — 隐私状态注解系统

```
#[program]
mod my_program {
    #[private]              // ← 这一行，变量变成隐私
    let balance: u64;

    #[private]
    let credit_score: u16;

    pub fn transfer(ctx, amount: u64) {
        // balance 是私有的,外部不可见
        // 但零知识证明保证了这里的状态更新是合法的
    }
}
```

- `#[private]`: 标注的变量状态不在链上明文存储，只在承诺树中存储承诺
- `#[selective(attrs = ">100", "whale")]`: 支持选择性披露条件
- `#[private(expires = "7d")]`: 支持时间窗口过期

#### P0 — ZK 电路自动编译器

- 输入: 标注了 `#[private]` 的 Solana 程序 Rust 代码
- 过程: 
  1. 提取所有 `#[private]` 变量和条件逻辑为 IR
  2. 自适应路由选择最佳 ZK 后端 (Groth16/Plonk/STARK)
  3. 编译为 Solana BPF 可执行格式 + 验证者程序
- 输出: Verifier Program + Client SDK (WASM 生成证明)

#### P0 — 链上 Verifier 程序

- 递归证明聚合: 10 笔隐私交易 = 1 次链上验证
- 批验证: 多个用户的证明批量验证
- 承诺树根更新: 维护全局私有状态默克尔树
- 选择性披露事件: 合规审计方接收的事件流

#### P0 — 私有状态承诺树

- Sparse Merkle Tree with versioned leaves
- 支持: 插入 / 更新 / 撤销 / 时间窗口过期
- 叶子结构: `(Nullifier, Commitment, Version, Expiry, SelectorMask)`
- 树根作为程序 PDA，全局一致

### 3.2 P1 功能

- **跨程序隐私组合调用**: 公开程序 ↔ 隐私程序互相调用，组合 ZK 证明
- **CLI 开发工具链**: `privy init` / `privy build` / `privy verify` / `privy deploy`
- **TypeScript SDK**: `@privy-svm/client` npm 包
- **Rust SDK**: `privy-svm` crate，供后端和脚本使用
- **证明浏览器**: 查看链上已发布的证明（匿名部分不可见）
- **Groth16 Ceremony 集成**: 安全参数设置的门户

### 3.3 P2 功能

- **隐私事件订阅**: 应用可以订阅"有新的隐私证明通过验证"事件
- **GDPR 合规档**: 数据被遗忘权的标准流程
- **多命名空间承诺树**: 不同应用可以有独立的私有状态空间
- **证明压缩 & 递归**: 支持将多步隐私计算压缩为单步链上验证
- **移动端 SDK**: React Native / Swift / Kotlin

---

## 4. 非功能需求

### 4.1 性能

| 指标 | 目标值 | 为什么要这个值 |
|---|---|---|
| ZK 证明生成 (浏览器/WASM) | ≤ 500ms | 用户体验不阻塞 |
| ZK 证明生成 (原生) | ≤ 100ms | 高频交易场景 |
| 链上验证 (Groth16) | ≤ 5ms (≈ 20K CU) | 低于 Solana 单笔交易预算 |
| 链上验证 (Plonk) | ≤ 3ms | 批量场景 |
| 递归聚合 10 个证明 | ≤ 15ms | 高并发场景 |
| 承诺树更新 | ≤ 1ms | 正常状态更新 |

### 4.2 安全

- 所有 ZK 电路开源，可独立审计
- Groth16 必须有可信设置 Ceremony
- 承诺树采用 Poseidon 哈希（ZK 友好）
- 防止双花: 每笔隐私操作有唯一 Nullifier
- 审计日志: 所有隐私操作的公开元数据上链
- 正式化验证: 核心电路用 Lean4 证明正确性

### 4.3 开发者体验

- CLOC 指标: 开发者实现第一个隐私程序的代码行数 ≤ 50 行
- TTFHW (Time To First Hello World): ≤ 5 分钟
- 文档: 完整 Cookbook + Examples + API Reference
- 错误信息: 编译器输出人类可读的错误提示

### 4.4 合规

- 支持选择性披露给审计方（不需要披露完整数据）
- 支持证明过期和手动撤销
- 用户始终控制自己的隐私数据
- 不存储用户的个人身份信息

---

## 5. 技术约束

| 约束 | 详情 |
|---|---|
| 链 | Solana Mainnet / Devnet |
| 程序编译目标 | Solana BPF (eBPF fork) |
| ZK 后端 | groth16-solana（Solana 定制版）+ Plonk + STARK |
| 哈希 | Poseidon (电路友好) + SHA256 (链上) |
| 默克尔树 | Sparse Merkle Tree, 256 层 |
| 前端框架 | Next.js 14 + React |
| 浏览器 ZK | WASM (arkworks-rs 编译) |
| 编程语言 | Rust (程序), TypeScript (前端/SDK), Solidity N/A |
| 性能预算 (CU) | 单笔交易 ≤ 200K CU，优先 ≤ 20K CU |

---

## 6. 成功指标

### 6.1 黑客松目标

| 指标 | 目标 |
|---|---|
| Demo 场景 | 3 个完整可交互的 Demo |
| 代码覆盖率 | 核心合约 100% |
| 演示流畅度 | 零报错，10 秒内完成演示流程 |
| 技术深度 | ZK 电路从 Rust 源代码编译到链上验证的完整链路 |

### 6.2 长期目标

| 指标 | 目标 |
|---|---|
| 开发者采用 | 黑客松后 3 个月内 50+ 程序使用 |
| TVL | 通过 Privy SVM 的隐私交易量 > $10M |
| 生态集成 | 5+ DeFi 协议集成 |

---

## 7. 竞品分析

| 方案 | 链 | 语言 | 隐私级别 | 优点 | 缺点 |
|---|---|---|---|---|---|
| **Aztec** | ETH L2 | Noir | 合约级 | 完整的隐私 L2 | 不是 Solana,新语言学习成本 |
| **Tornado Cash** | ETH | N/A | 交易级 | 简单易用 | 被制裁,纯匿名不可合规 |
| **Elusiv** | Solana | N/A | 交易级 | Solana 原生 | 只做交易隐私,不解决通用计算隐私 |
| **Light Protocol** | Solana | Rust | 合约级 | Solana ZK 基础设施 | 不是隐私虚拟机,需要开发者自己搭电路 |
| **Privy SVM** | Solana | Rust | **字段级** | **零学习成本,注解驱动,字段级粒度** | 新项目,需要生态建设 |

**Privy SVM 的护城河**:
- 字段级隐私粒度（竞争对手只有交易级或合约级）
- Rust 原生注解（竞争对手要求学新语言）
- 公开 ↔ 隐私原生混合调用（竞争对手需要复杂桥接）
- Solana 的并行执行优势（ETH 上的方案受限于顺序执行）

---

## 8. 交付计划

### Phase 1: 黑客松 MVP (2 周)

- [ ] ZK 电路编译器核心（注解 → R1CS）
- [ ] Solana Verifier Program v1
- [ ] 承诺树合约
- [ ] 浏览器端证明生成 (WASM)
- [ ] 3 个 Demo: ZK 扑克 / 隐私暗池 / 隐私投票
- [ ] 前端展示页面

### Phase 2: 生产化 (3 个月)

- [ ] Groth16 Ceremony
- [ ] 多证明后端自适应路由
- [ ] CLI 工具链
- [ ] TypeScript / Rust SDK v1
- [ ] 安全审计
- [ ] Devnet 上线

### Phase 3: 生态建设 (6 个月)

- [ ] 5+ DeFi 协议集成
- [ ] 移动端 SDK
- [ ] 正式化验证 (Lean4)
- [ ] Mainnet 上线
- [ ] 开发者 Grants 计划

---

## 9. 风险 & 缓解

| 风险 | 等级 | 缓解措施 |
|---|---|---|
| Groth16 可信设置参数泄露 | 高 | 社区 Multi-Party Computation Ceremony |
| ZK 证明生成太慢 | 中 | WASM 编译优化 + 自适应路由选最快后端 |
| Solana BPF 不支持 ZK 复杂计算 | 中 | 链上只做验证,证明生成离线 |
| 监管风险 | 中 | 选择性披露 + GDPR 支持,不做纯匿名混币 |
| 竞争方案出现 | 低 | Rust 注解方案是独特技术壁垒,其他人需要开发者学新语言 |

---

## 10. 附录

### A. 术语表

| 术语 | 定义 |
|---|---|
| ZK (Zero-Knowledge) | 零知识证明,证明某事为真而不泄露信息 |
| SVM | Solana Virtual Machine |
| R1CS | Rank-1 Constraint System, ZK 电路的数学表示 |
| Groth16 | 最小证明体积的 ZK 方案 |
| Plonk | 无需可信设置的通用 ZK 方案 |
| STARK | 不需要可信设置的可扩展 ZK 方案 |
| Nullifier | 防止双花的唯一标识符 |
| Commitment | 对值的加密承诺 |
| SelectorMask | 控制哪些属性可被选择性披露的位掩码 |
| CU | Compute Units, Solana 链上计算单位 |
