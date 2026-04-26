# 技术设计文档: Privy SVM

> Architecture & Technical Design | Version 1.0 | 2026-04-26

---

## 目录

1. [架构总览](#1-架构总览)
2. [ZK 电路编译器](#2-zk-电路编译器-privycc)
3. [私有状态承诺树](#3-私有状态承诺树-ptre)
4. [Solana 链上验证者程序](#4-solana-链上验证者程序)
5. [自适应 ZK 后端路由](#5-自适应-zk-后端路由)
6. [跨程序隐私组合调用](#6-跨程序隐私组合调用)
7. [客户端开发 SDK](#7-客户端开发-sdk)
8. [前端架构](#8-前端架构)
9. [安全模型 & 威胁分析](#9-安全模型--威胁分析)
10. [部署架构](#10-部署架构)
11. [Demo 场景技术方案](#11-demo-场景技术方案)

---

## 1. 架构总览

### 1.1 分层架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    用户代理层 (User Agent)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ React 前端   │  │ CLI 工具链   │  │ 移动端 SDK (未来)    │  │
│  │ (Next.js 14) │  │ (privy-cli)  │  │                      │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘  │
├─────────┼─────────────────┼──────────────────────┼──────────────┤
│         ▼                  ▼                       ▼              │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │            客户端证明层 (Client Proof Layer)               │   │
│  │  ┌─────────────┐  ┌────────────┐  ┌──────────────────┐  │   │
│  │  │ WASM 证明   │  │ Rust 原生  │  │ 选择性披露       │  │   │
│  │  │ 生成器      │  │ 证明生成器 │  │ 构建器           │  │   │
│  │  └─────────────┘  └────────────┘  └──────────────────┘  │   │
│  └─────────────────────────┬────────────────────────────────┘   │
├────────────────────────────┼────────────────────────────────────┤
│                            ▼                                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │            编译器层 (Compiler Layer) — PrivyCC             │   │
│  │  ┌───────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │   │
│  │  │ 注解解析器│→│ IR 生成器│→│ 约束生成 │→│后端路由 │  │   │
│  │  └───────────┘  └──────────┘  └──────────┘  └────┬───┘  │   │
│  └──────────────────────────────────────────────────┼────────┘   │
├─────────────────────────────────────────────────────┼────────────┤
│                                                     ▼            │
│  ┌──────────────────┐ ┌────────────┐ ┌────────────────────────┐ │
│  │ Groth16 后端     │ │ Plonk 后端 │ │ STARK 后端             │ │
│  │ (小计算, 快验证) │ │ (中计算)    │ │ (大计算, 高吞吐)       │ │
│  └──────────────────┘ └────────────┘ └────────────────────────┘ │
├──────────────────────────────────────────────────────────────────┤
│                    链上执行层 (On-Chain)                          │
│  ┌─────────────┐  ┌──────────────────┐  ┌───────────────────┐  │
│  │ Verifier    │  │ 承诺树管理器     │  │ 隐私事件发射器    │  │
│  │ Program     │  │ (PSTree Manager) │  │ (Event Emitter)   │  │
│  └─────────────┘  └──────────────────┘  └───────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │             私有状态承诺树 (Solana PDA)                    │   │
│  │  ┌────────────────────────────────────────────────────┐  │   │
│  │  │  Sparse Merkle Tree (256 depth)                    │  │   │
│  │  │  Leaf = (Nullifier, Commitment, Version, Expiry,   │  │   │
│  │  │          SelectorMask)                             │  │   │
│  │  └────────────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

### 1.2 数据流（一笔隐私交易的完整生命周期）

```
用户浏览器                    链下编译器/证明层               Solana 链上
───────                      ────────────────               ──────────

1. 用户操作
   (如: 隐私转账)
        │
        ▼
2. 构造隐私交易              3. 编译器生成 ZK 电路
   {                                │
    from: [nullifier]               ▼
    to: [commitment]              4. 生成 Groth16 证明
    amount: [hidden]               proof = π₁, π₂, π₃
    proof: [待生成]                     │
   }                                    ▼
        │                         5. 浏览器端完成
        │                            证明生成 (~500ms)
        ▼
6. 组装完整交易 ──────────────────▶ 7. Solana 交易执行:
   tx = {                                verify(proof, public_inputs)  ←~5ms
     ix1: verifier.verify(proof)         check_nullifier_not_spent()   ←~1ms
     ix2: pstree.insert_leaf(comm)      insert_commitment_tree()      ←~1ms
     ix3: app.execute(public_out)       emit PrivacyEvent              ←~1ms
   }                                   ✓ 交易确认 (~10ms)
        │
        ▼
8. 用户收到确认
   ✅ "已发送 1 SOL (隐私)"
```

### 1.3 关键技术指标

| 指标 | 目标 | 瓶颈 |
|---|---|---|
| 证明生成 (浏览器) | ≤ 500ms | WASM 执行速度 |
| 证明生成 (原生) | ≤ 100ms | Rust 原生执行 |
| 链上验证 | ≤ 20K CU | Solana BPF 限制 |
| 递归聚合吞吐 | 10 proofs/s | 链上计算量 |
| 承诺树操作 | ≤ 5K CU | Merkle 路径长度 |

---

## 2. ZK 电路编译器 (PrivyCC)

### 2.1 编译管线

```
Rust 源码               IR (中间表示)           约束系统            链上代码
────────              ──────────────          ────────           ────────

#[private]       →    ┌──────────────┐     ┌──────────────┐   ┌──────────────┐
let x: u64;           │ Annotation   │     │ Constraint   │   │ Verifier     │
                      │ Parser       │ →   │ Generator    │ → │ Program      │
if x > 100 {          │ (proc-macro) │     │ (IR → R1CS)  │   │ (Solana BPF) │
    transfer()        └──────┬───────┘     └──────┬───────┘   └──────────────┘
}                            │                     │
                    ┌────────▼───────┐     ┌───────▼────────┐
                    │ Program IR     │     │ Circuit        │
                    │ • 私有变量列表 │ →   │ • R1CS 矩阵    │
                    │ • 条件分支 DAG │     │ • Plonk 排列   │
                    │ • 约束图       │     │ • STARK trace  │
                    └────────────────┘     └────────────────┘
```

### 2.2 注解语法规范

```rust
use privy_svm::prelude::*;

#[program]
mod my_program {
    // 基本私有变量
    #[private]
    let balance: u64;

    // 选择式披露 (生成证明时可选择公开哪些属性)
    #[private(selective(lower_bound, category))]
    let credit_score: CreditInfo {
        score: u64,           // 始终隐藏
        lower_bound: u64,     // 可公开 "score > 500"
        category: String,     // 可公开 "prime"
    };

    // 带过期的私有变量 (7天后证明自动失效)
    #[private(expires = "7d")]
    let kyc_result: KYCResult;

    // 可撤销变量 (用户可随时吊销证明)
    #[private(revocable)]
    let access_token: TokenID;

    // 公开变量不变 (完全透明的状态)
    let total_supply: u64;
    pub admin: Pubkey;

    // 隐私约束
    #[constraint]
    fn enforce_balance(ctx: Context<Transfer>) -> Result<()> {
        // 这段逻辑会被编译为 ZK 电路的一部分
        require!(ctx.accounts.from.balance >= amount, ErrorCode::Insufficient);
        Ok(())
    }
}
```

### 2.3 IR 设计

```rust
/// 程序中间表示
struct ProgramIR {
    /// 私有变量声明
    private_variables: Vec<PrivateVariable>,
    /// 公开变量声明
    public_variables: Vec<PublicVariable>,
    /// 约束条件 (由 #[constraint] 注解的函数生成)
    constraints: Vec<Constraint>,
    /// 状态转换 (写操作)
    state_transitions: Vec<StateTransition>,
    /// 披露策略 (每个私有变量的选择性披露配置)
    disclosure_policies: Vec<DisclosurePolicy>,
}

struct PrivateVariable {
    name: String,
    ty: VariableType,
    visibility: Visibility,        // FullyPrivate | Selective { attrs: Vec<String> }
    expiry: Option<Duration>,
    revocable: bool,
}

enum Visibility {
    FullyPrivate,                  // 完全不可见
    Selective { attributes: Vec<String> },  // 可选择披露的属性名列表
    TimedWindow { duration: Duration },     // 时间窗口后自动公开
}

struct Constraint {
    id: String,
    kind: ConstraintKind,
    expr: ConstraintExpr,          // AST of the constraint expression
}

enum ConstraintKind {
    RangeCheck { min: u64, max: u64 },
    BalanceCheck { account: String, amount: u64 },
    MerkleProof,
    SignatureCheck,
    Custom { name: String },
}

struct StateTransition {
    old_state: Commitment,
    new_state: Commitment,
    nullifier: Nullifier,
    transition_type: TransitionType,
}

enum TransitionType {
    Update,     // in-place 更新
    Transfer,   // from → to
    Create,     // 新建私有状态
    Destroy,    // 销毁私有状态 (带 nullifier)
    Revoke,     // 用户主动吊销
    Expire,     // 时间到期自动失效
}
```

### 2.4 编译器命令行接口

```bash
# 1. 初始化 Privy SVM 程序项目
privy init my_program

# 生成的目录结构:
# my_program/
# ├── Cargo.toml          ← 添加 privy-svm 依赖
# ├── Xargo.toml          ← BPF 编译配置
# ├── src/
# │   ├── lib.rs          ← 你的程序代码
# │   ├── constraints.rs  ← 约束定义
# │   └── state.rs        ← 状态定义
# ├── privy.toml          ← Privy SVM 配置
# └── tests/
#     └── integration.rs

# 2. 编译: 生成 ZK 电路 + Verifier Program + Client SDK
privy build

# 输出:
# target/
# ├── deploy/
# │   └── my_program.so              ← Verifier + App Logic
# ├── circuit/
# │   ├── r1cs.bin                   ← R1CS 约束矩阵
# │   ├── proving_key.bin            ← Groth16 证明密钥
# │   └── verification_key.bin       ← Groth16 验证密钥
# ├── client/
# │   ├── proof_generator.wasm       ← 浏览器用 WASM
# │   ├── proof_generator.js         ← JS 封装
# │   └── types.d.ts                 ← TypeScript 类型
# └── rust/
#     └── lib.rs                     ← Rust SDK 绑定

# 3. 部署到 Solana (自动处理 Verifier Program 的部署)
privy deploy --network devnet

# 4. 验证本地生成的证明是否合法
privy verify --proof proof.bin --public-inputs inputs.json
```

---

## 3. 私有状态承诺树 (PSTree)

### 3.1 数据结构

```
Sparse Merkle Tree, 256 depth
Hash: Poseidon-128 (ZK-friendly, ~30x faster than SHA256 in circuit)

         ┌───── Root (PDA) ─────┐
         │ 0xa1b2c3d4...        │
         │ Stored on-chain      │
         └──────────┬───────────┘
           ┌────────┴────────┐
        ┌──○──┐           ┌──○──┐
        │ Hash │           │ Hash │
        └──┬───┘           └──┬───┘
      ┌────┴────┐         ┌───┴────┐
     ┌○┐      ┌○┐        ┌○┐     ┌○┐
     │H│      │H│  ...   │H│ ... │H│
     └┬┘      └┬┘        └┬┘     └┬┘
      [Leaf]     [Leaf]     [Leaf]    [Leaf]
       n=5        n=7        n=100     n=255
```

### 3.2 叶子节点结构

```rust
/// 私有状态树的叶子节点
struct LeafNode {
    /// 版本号 — 每次更新递增, 旧版本证明自动失效
    version: u64,

    /// 承诺值 — 对私有字段的哈希承诺
    /// commitment = PoseidonHash(value | blinding_factor | program_id | namespace)
    commitment: [u8; 32],

    /// 空标识符 — 当承诺被消耗/撤销时, 公开此值防止双花
    nullifier: Nullifier,

    /// 过期时间戳 — unix timestamp, 0 = 永不过期
    expiry_timestamp: i64,

    /// 选择性披露掩码
    /// bit 0 → attrs[0] 可被公开
    /// bit 1 → attrs[1] 可被公开
    /// 全 0 = 完全不可公开
    selector_mask: u64,

    /// 公开输入哈希 (用于验证时传入)
    /// 由用户选择公开哪些属性的值, PoseidonHash(公开值列表)
    public_inputs_hash: [u8; 32],
}

/// 空标识符生成:
/// nullifier = PoseidonHash(private_key | leaf_index | version)
/// 公开 nullifier 后,该版本的承诺不能再被使用
pub struct Nullifier(pub [u8; 32]);

impl LeafNode {
    /// 计算叶子节点的哈希值
    fn hash(&self) -> [u8; 32] {
        PoseidonHash::hashv(&[
            &self.version.to_le_bytes(),
            &self.commitment,
            &self.nullifier.0,
            &self.expiry_timestamp.to_le_bytes(),
            &self.selector_mask.to_le_bytes(),
            &self.public_inputs_hash,
        ])
    }
}
```

### 3.3 树操作

```rust
/// 私有状态树管理器 PDA
/// seeds = [b"pstree", program_id.as_ref(), namespace.as_ref()]
#[account]
pub struct PSTreeAccount {
    /// 当前根哈希
    root: [u8; 32],

    /// 树中叶子节点总数
    leaf_count: u64,

    /// 命名空间 (一个程序可管理多个独立的树)
    namespace: u64,

    /// 上一次更新时的 slot
    last_updated_slot: u64,

    /// nullifier 黑名单 (标记已消耗的承诺)
    nullifier_registry: Vec<NullifierPair>,

    /// 已过期的叶子索引列表 (定期清理用)
    expired_leaves: Vec<u64>,
}

/// 树操作指令
enum PSTreeInstruction {
    /// 插入新承诺 — 创建一条新的私有状态
    Insert {
        commitment: [u8; 32],
        merkle_proof: MerkleProof,
        namespace: u64,
    },

    /// 更新承诺 — 修改现有私有状态 (带旧的 nullifier)
    Update {
        old_nullifier: Nullifier,
        new_commitment: [u8; 32],
        merkle_proof: MerkleProof,
    },

    /// 标记为已消耗 (用 nullifier 销毁承诺)
    Consume {
        nullifier: Nullifier,
    },

    /// 用户主动撤销
    Revoke {
        nullifier: Nullifier,
        authorization_signature: Signature,  // 用户签名证明所有权
    },

    /// 清理过期承诺 (任何人可调用,获得小奖励)
    ExpireCleanup {
        leaf_indices: Vec<u64>,
        merkle_proofs: Vec<MerkleProof>,
    },
}
```

### 3.4 nullifier 双花防护

```
方案: Nullifier 注册表 + 查找表

1. 每笔隐私操作暴露一个 nullifier
2. Verifier 检查 nullifier 是否已存在于 nullifier_registry
3. 如果存在 → 拒绝交易 (双花)
4. 如果不存在 → 接受,将 nullifier 加入注册表

nullifier_registry 存储为 Sparse Merkle Tree → O(log n) 查找
当前方案: 前 100K nullifier 用位图(bloom filter), 之后用 SMT

存储优化:
- 每个 nullifier 32 bytes
- 100 万 nullifier ≈ 32 MB (用 SMT 存, 按需加载)
- 定期清理: 已验证且不再需要的叶子节点的 nullifier 可移入 cold storage
```

---

## 4. Solana 链上验证者程序

### 4.1 Verifier Program 架构

```rust
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    pubkey::Pubkey,
    program_error::ProgramError,
};

entrypoint!(process_instruction);

/// 验证者程序入口
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = VerifierInstruction::unpack(instruction_data)?;

    match instruction {
        VerifierInstruction::VerifyProof {
            proof_type,
            proof_data,
            public_inputs,
        } => verify_proof(program_id, accounts, proof_type, proof_data, public_inputs),

        VerifierInstruction::BatchVerify {
            proofs,
        } => batch_verify(program_id, accounts, proofs),

        VerifierInstruction::VerifyAndUpdateState {
            proof_data,
            public_inputs,
            state_transition,    // 状态转换的公开部分
        } => verify_and_update_state(
            program_id,
            accounts,
            proof_data,
            public_inputs,
            state_transition,
        ),
    }
}

/// 核心: 验证单个 ZK 证明
fn verify_proof(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    proof_type: ProofType,
    proof_data: Vec<u8>,
    public_inputs: Vec<[u8; 32]>,
) -> ProgramResult {
    // 1. 解包验证密钥 (从程序数据段加载)
    let vk = load_verification_key(program_id, proof_type)?;

    // 2. 执行 ZK 验证 (目标: ≤ 20K CU)
    let verified = match proof_type {
        ProofType::Groth16 => {
            // 椭圆曲线配对 + 标量乘法
            // ~5ms (≈15K CU on Solana)
            groth16::verify(&vk, &proof_data, &public_inputs)?
        }
        ProofType::Plonk => {
            // 多项式承诺验证
            // ~3ms (≈10K CU)
            plonk::verify(&vk, &proof_data, &public_inputs)?
        }
        ProofType::STARK => {
            // FRI 验证
            // ~10ms (≈30K CU)
            stark::verify(&vk, &proof_data, &public_inputs)?
        }
    };

    if !verified {
        return Err(ProgramError::Custom(ErrorCode::InvalidProof as u32));
    }

    // 3. 发射验证成功事件
    emit_event(PrivacyEvent::ProofVerified {
        proof_type,
        public_inputs_hash: hash_public_inputs(&public_inputs),
    })?;

    Ok(())
}

/// 批量验证多个证明 (递归聚合)
fn batch_verify(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    proofs: Vec<BatchProof>,
) -> ProgramResult {
    // 用递归 SNARK 聚合多个证明
    // 10 个证明 → 1 次链上配对检查
    // ~15ms (≈20K CU) 而不是 10×5ms

    let mut accumulator = RecursiveProof::empty();

    for proof in proofs {
        accumulator = recursive_aggregate(
            &accumulator,
            &proof.data,
            &proof.public_inputs,
        )?;
    }

    verify_recursive_proof(&accumulator)
}
```

### 4.2 指令格式

```rust
enum VerifierInstruction {
    /// 单证明验证
    VerifyProof {
        proof_type: ProofType,       // 1 byte
        proof_data: Vec<u8>,          // Groth16: ~128 bytes
        public_inputs: Vec<[u8; 32]>, // 1-10 inputs
    },

    /// 批量验证
    BatchVerify {
        proofs: Vec<BatchProof>,      // 每条 ~160 bytes
    },

    /// 验证并更新私有状态树
    VerifyAndUpdateState {
        proof_data: Vec<u8>,
        public_inputs: Vec<[u8; 32]>,
        state_transition: StateTransitionData,
    },
}

/// CU 消耗估算
/// | 指令                    | CU 估算 |
/// |─────────────────────────|─────────|
/// | VerifyProof (Groth16)   | ~15,000 |
/// | VerifyProof (Plonk)     | ~10,000 |
/// | VerifyProof (STARK)     | ~30,000 |
/// | BatchVerify (10 proofs) | ~20,000 |
/// | VerifyAndUpdateState    | ~25,000 |
```

### 4.3 错误处理

```rust
#[repr(u32)]
pub enum ErrorCode {
    // ZK 验证错误
    InvalidProof = 0x1,
    ProofVerificationFailed = 0x2,
    InvalidPublicInputs = 0x3,
    UnsupportedProofType = 0x4,

    // 状态树错误
    NullifierAlreadySpent = 0x10,   // 双花检测
    CommitmentAlreadyExists = 0x11,
    MerkleProofInvalid = 0x12,
    LeafNotFound = 0x13,
    LeafExpired = 0x14,             // 在 expiry_timestamp 之后使用

    // 授权错误
    UnauthorizedRevoke = 0x20,      // 非拥有者试图撤销
    InvalidSignature = 0x21,

    // 系统错误
    BatchTooLarge = 0x30,           // 批量证明数量超过上限
    InsufficientCUBudget = 0x31,    // CU 预算不足
}
```

---

## 5. 自适应 ZK 后端路由

### 5.1 路由决策树

```
                    开始构建证明
                          │
                          ▼
                ┌─ 约束数量 < 500? ─┐
                │                   │
               YES                  NO
                │                   │
                ▼                   ▼
          Groth16             约束 < 5000?
          最小证明                  │
          最快验证           ┌─────┴─────┐
                            YES          NO
                             │            │
                             ▼            ▼
                           Plonk        STARK
                         无信任设置    最大规模
                         中等证明      最快速生成
```

### 5.2 路由策略表

```rust
/// ZK 后端选择器
enum ZkBackend {
    Groth16,    // 小型约束 (≤500), 最小链上足迹
    Plonk,      // 中型约束 (500-5000), 无需可信设置
    STARK,      // 大型约束 (5000+), 最大吞吐量
}

struct RoutingDecision {
    backend: ZkBackend,
    estimated_proof_time_ms: u64,      // 浏览器端生成时间预估
    estimated_verification_cu: u64,    // 链上验证 CU 预估
    proof_size_bytes: usize,           // 证明大小
}

impl ProgramIR {
    /// 自动选择最优 ZK 后端
    fn select_backend(&self) -> RoutingDecision {
        let constraint_count = self.constraints.len();
        let variable_count = self.private_variables.len();

        match constraint_count {
            0..=500 => {
                // Groth16: 最小证明 (128 bytes), 最快验证
                RoutingDecision {
                    backend: ZkBackend::Groth16,
                    estimated_proof_time_ms: 500,
                    estimated_verification_cu: 15_000,
                    proof_size_bytes: 128,
                }
            }
            501..=5000 => {
                // Plonk: 不需要可信设置, 中等证明 (400 bytes)
                // 当需要频繁升级电路时优先选择
                let needs_frequent_update = self.has_selective_disclosure();
                if needs_frequent_update {
                    RoutingDecision {
                        backend: ZkBackend::Plonk,
                        estimated_proof_time_ms: 800,
                        estimated_verification_cu: 10_000,
                        proof_size_bytes: 400,
                    }
                } else {
                    // 不需要频繁更新 → 用 Groth16 做递归聚合
                    RoutingDecision {
                        backend: ZkBackend::Groth16,
                        estimated_proof_time_ms: 500,
                        estimated_verification_cu: 15_000,
                        proof_size_bytes: 128,
                    }
                }
            }
            _ => {
                // STARK: 不需要可信设置, 证明大 (10KB+), 但生成快
                RoutingDecision {
                    backend: ZkBackend::STARK,
                    estimated_proof_time_ms: 300,
                    estimated_verification_cu: 30_000,
                    proof_size_bytes: 10_240,
                }
            }
        }
    }
}
```

### 5.3 三种后端的链上成本对比

```
                  Groth16          Plonk            STARK
─────────────────────────────────────────────────────────
证明大小          128 bytes       400 bytes       10 KB
验证 CU           15,000          10,000          30,000
验证时间          5ms             3ms             10ms
可信设置          需要            不需要          不需要
电路大小限制      中              大              极大
递归聚合          支持            支持            复杂
─────────────────────────────────────────────────────────
适用场景          支付/转账      身份认证/投票   游戏/大数据
```

---

## 6. 跨程序隐私组合调用

### 6.1 核心概念

```
这是 Privy SVM 最具创新性的设计:
公开 Solana 程序 ↔ 隐私 Solana 程序 可以直接互相调用,
组合 ZK 证明,而不需要任何桥接。

传统方案 (Aztec):
  公开合约 → 桥接合约 → 隐私合约 (异步,有延迟,有风险)

Privy SVM:
  公开程序 → CPI → 隐私程序 (同步,原子,零信任假设)
```

### 6.2 设计

```rust
/// 跨程序指令接口
#[derive(Accounts)]
pub struct CrossProgramPrivacyCall<'info> {
    /// 调用方 — 可以是任何 Solana 程序
    pub caller_program: AccountInfo<'info>,

    /// Privy SVM 验证者程序
    pub verifier_program: Program<'info, VerifierProgram>,

    /// 私有状态树 PDA
    #[account(seeds = [b"pstree", verifier_program.key().as_ref()])]
    pub pstree: Account<'info, PSTreeAccount>,

    /// 隐私证明 (传递到 Verifier)
    pub proof_account: AccountInfo<'info>,

    /// 被调用的目标隐私程序
    pub target_private_program: AccountInfo<'info>,
}

/// 公开程序调用隐私程序的流程
///
/// 公开 DEX 程序:
///   1. 用户说:"用我的信用分借款"
///   2. DEX 调用 Privy SVM Verifier 验证用户的信用分证明
///   3. Verifier 验证 proof → 返回 "信用分 > 700" (不泄露具体分数)
///   4. DEX 基于验证结果执行借款
///
/// CPI 调用链: DEX → Verifier.VerifyProof → DEX 收到结果 → DEX.execute_lend
pub fn cross_program_privacy_call<'info>(
    ctx: Context<'_, '_, '_, 'info, CrossProgramPrivacyCall<'info>>,
    proof_data: Vec<u8>,
    public_query: QuerySpec,  // "信用分 > 700" 查询
) -> ProgramResult {
    // 1. 构造 CPI 调用: caller → verifier.verify
    let verify_ix = Instruction {
        program_id: ctx.accounts.verifier_program.key(),
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.pstree.key(), false),
            AccountMeta::new_readonly(ctx.accounts.proof_account.key(), false),
        ],
        data: verifier::instruction::VerifyProof {
            proof_data: proof_data.clone(),
            public_inputs: public_query.to_public_inputs(),
        }.serialize(),
    };

    // 2. 执行 CPI (隐私验证)
    solana_program::program::invoke(
        &verify_ix,
        &[
            ctx.accounts.pstree.to_account_info(),
            ctx.accounts.proof_account.to_account_info(),
            ctx.accounts.verifier_program.to_account_info(),
            ctx.accounts.caller_program.to_account_info(),
        ],
    )?;

    // 3. Verifier 验证成功后, 公开程序继续执行
    //    (这里 DEX 知道了 "信用分 > 700" 但不泄露具体分数)
    let verified_attributes = parse_verification_result()?;

    if verified_attributes.credit_score_above(700) {
        // 4. 执行放贷
        execute_lend(ctx, verified_attributes.loan_amount)
    } else {
        Err(ErrorCode::CreditScoreTooLow)
    }
}
```

### 6.3 组合调用原子性保证

```
Solana CPI 保证原子性:
  如果 Verifier.verify 成功   → DEX 借款执行
  如果 Verifier.verify 失败   → 整个交易回滚
  如果 DEX 借款执行失败       → 整个交易回滚 (证明不会留下孤立状态)

这是 Solana 相对于 Ethereum L2 的天然优势:
所有操作在同一条链上,同一个块中,同一个事务中。
```

---

## 7. 客户端开发 SDK

### 7.1 JavaScript/TypeScript SDK (`@privy-svm/client`)

```typescript
// npm install @privy-svm/client

import { PrivyClient, ProofBuilder, SelectiveDisclosure } from '@privy-svm/client';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';

// 初始化客户端
const client = new PrivyClient({
    connection: new Connection('https://api.devnet.solana.com'),
    programId: new PublicKey('PrivyV1111111111111111111111111111111111111'),
});

// 示例 1: 隐私转账
async function privacyTransfer(
    from: Keypair,
    to: PublicKey,
    amount: number,
) {
    // 构建隐私证明 (浏览器端, WASM)
    const proofBuilder = new ProofBuilder({
        namespace: 'transfer',
        backend: 'g16',  // 手动指定或 'auto'
    });

    // 隐藏值
    proofBuilder.addPrivate({
        from_balance: 100,          // 隐藏起始余额
        transfer_amount: amount,    // 隐藏转账金额
    });

    // 公开值
    proofBuilder.addPublic({
        to_address: to.toBytes(),
        timestamp: Date.now(),
    });

    // 添加约束: 余额 >= 转账金额
    proofBuilder.constrain('from_balance >= transfer_amount');

    // 生成证明 (浏览器端 ~500ms)
    const proof = await proofBuilder.generate();
    console.log('Proof generated:', proof.hex());

    // 上链
    const tx = await client.verifyAndExecute(proof, from);
    const signature = await client.sendTransaction(tx);
    await client.confirmTransaction(signature);

    console.log('Privacy transfer complete:', signature);
}

// 示例 2: 选择性披露
async function selectiveDisclosure() {
    const disclosure = new SelectiveDisclosure({
        // 我只证明"信用分 > 700",不暴露具体分数
        prove: {
            credit_score: {
                type: 'range',
                operator: 'gt',
                value: 700,
            },
            verified: true,     // "已通过 KYC"
        },
        // 不暴露: credit_score 的具体值, address, name
    });

    const proof = await disclosure.generateProof();
    const result = await client.verifyDisclosure(proof);

    // result = { credit_score_gt_700: true, verified: true }
    // 协议方只知道这两个布尔值,不知任何具体数值
}
```

### 7.2 Solana Wallet 适配器集成

```typescript
import { useWallet } from '@solana/wallet-adapter-react';
import { usePrivy } from '@privy-svm/react';

function PrivacyDApp() {
    const wallet = useWallet();
    const privy = usePrivy();

    const executePrivacyTx = async () => {
        if (!wallet.connected) return;

        // Builder 链式调用
        const tx = await privy
            .builder('my_private_program')
            .private({ key: 'balance', value: 1000 })
            .public({ key: 'recipient', value: recipientPubkey })
            .constrain('balance >= amount')
            .selective(['recipient'])  // 只公开 recipient,其他全隐藏
            .build();

        await wallet.sendTransaction(tx, privy.connection);
    };

    return <button onClick={executePrivacyTx}>Send Privacy Tx</button>;
}
```

### 7.3 Rust SDK (`privy-svm` crate)

```rust
// Cargo.toml:
// [dependencies]
// privy-svm = "0.1.0"

use privy_svm::prelude::*;
use solana_sdk::signature::Keypair;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PrivyClient::new(
        "https://api.devnet.solana.com",
        privy_svm::id(),
    );

    let proof = ProofBuilder::new("transfer", Backend::Groth16)
        .private("from_balance", 1000u64)?
        .private("amount", 50u64)?
        .public("to_address", recipient_pubkey)?
        .constrain(BalanceCheck::gte("from_balance", "amount"))?
        .generate()?;  // 原生 Rust, ~100ms

    let tx = client.verify_and_execute(proof, &signer_keypair)?;
    client.confirm_transaction(&tx)?;

    Ok(())
}
```

---

## 8. 前端架构

### 8.1 技术栈

```
Next.js 14 (App Router) + React 19
├── TypeScript 5.x
├── Tailwind CSS 4 + shadcn/ui
├── Framer Motion (动画/交互)
├── @solana/web3.js + @solana/wallet-adapter
├── @privy-svm/client (自己的 SDK)
├── Zustand (轻量状态管理)
└── WASM (浏览器内 ZK 证明生成)
```

### 8.2 前端路由 & 组件树

```
/                            → Landing Page (产品展示,架构动画)
├── /demo/poker              → ZK 扑克 Demo
├── /demo/darkpool           → 隐私暗池 Demo
├── /demo/vote               → 隐私投票 Demo
├── /docs                    → 文档站
│   ├── /docs/cookbook       → Cookbook 入门
│   ├── /docs/api            → API Reference
│   └── /docs/architecture   → 架构说明
├── /prove                   → 证明浏览器 (查看链上证明)
├── /playground              → 在线 Playground (写 Rust → 编译 → 生成证明)
└── /dashboard               → 用户仪表盘 (授权管理, 撤销, 历史)
```

### 8.3 核心组件

```typescript
// 组件树
<App>
  <SolanaWalletProvider>
    <PrivyClientProvider>
      <Navbar>
        <Logo />
        <WalletButton />
        <NetworkSelector />
      </Navbar>

      <main>
        {/* Landing Page */}
        <HeroSection>
          <AnimatedPrivacyVisual />   {/* ZK 证明生成动画 */}
          <CTAButton />
          <ArchitectureDiagram />     {/* 交互式架构图 */}
        </HeroSection>

        {/* ZK 扑克 Demo */}
        <PokerDemo>
          <CardTable>
            <PlayerHand privacy={true} />   {/* 手牌隐藏 */}
            <CommunityCards />
            <OpponentHand privacy={true} /> {/* 对手手牌隐藏 */}
          </CardTable>
          <ProofStatus>
            <VerifyBadge status="verified" />
            <ProofDetails />
          </ProofStatus>
          <GameLog />                       {/* 公开的游戏日志 */}
        </PokerDemo>

        {/* 隐私暗池 Demo */}
        <DarkPoolDemo>
          <OrderBook privacy={true} />      {/* 暗池订单簿 */}
          <TradeExecution />
          <PrivacyLevel>max</PrivacyLevel>
          <SettlementProof />
        </DarkPoolDemo>

        {/* 隐私投票 Demo */}
        <VoteDemo>
          <VotingBallot privacy={true} />   {/* 投票内容隐藏 */}
          <VoterEligibilityProof />         {/* 资格证明 */}
          <TallyBoard />                    {/* 公开结果 */}
        </VoteDemo>

        {/* 证明浏览器 */}
        <ProofExplorer>
          <ProofList>
            <ProofCard>
              <ProofType type="Groth16" />
              <PublicInputsHash />
              <Timestamp />
              <VerificationStatus />
            </ProofCard>
          </ProofList>
        </ProofExplorer>

        {/* Playground */}
        <Playground>
          <CodeEditor language="rust" />   {/* 在线写 Rust */}
          <CompileButton />
          <CircuitViewer />                {/* R1CS 可视化 */}
          <ProofGenerator />               {/* 测试生成证明 */}
          <ProofStats />                   {/* 性能统计 */}
        </Playground>
      </main>

      <Footer />
    </PrivyClientProvider>
  </SolanaWalletProvider>
</App>
```

### 8.4 数据流 (Frontend)

```
用户操作
    │
    ▼
React Component  ──────────────────────────────────────
    │
    ├── 1. 构造隐私数据 (PrivyProvider)
    │       ↓
    ├── 2. WASM Worker 生成 ZK 证明 (~500ms)
    │       ↓  ← 展示加载动画 (Framer Motion)
    ├── 3. 组装 Solana 交易
    │       ↓
    ├── 4. Wallet Adapter 签名
    │       ↓
    ├── 5. 发送交易到 RPC
    │       ↓  ← 展示确认动画
    └── 6. 更新 UI (Zustand store)
            ↓
        展示验证成功 / 交易确认
```

### 8.5 动画设计 (Framer Motion)

```typescript
// ZK 证明生成的视觉效果

// Phase 1: 数据加密 (粒子聚拢)
<motion.div
    initial={{ opacity: 0, scale: 0.5 }}
    animate={{ opacity: 1, scale: 1 }}
    transition={{ duration: 0.3 }}
>
    <ParticleEncryption />
</motion.div>

// Phase 2: 电路计算 (旋转矩阵)
<motion.div
    animate={{ rotate: 360 }}
    transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
>
    <CircuitMatrix />
</motion.div>

// Phase 3: 证明生成 (脉冲效果)
<motion.div
    animate={{ boxShadow: ["0 0 0 0px #9945FF", "0 0 0 20px transparent"] }}
    transition={{ duration: 1.5, repeat: Infinity }}
>
    <ProofGenerated />
</motion.div>

// Phase 4: 上链确认 (绿色波纹)
<motion.div
    initial={{ scale: 0 }}
    animate={{ scale: 1 }}
    transition={{ type: "spring", stiffness: 400, damping: 20 }}
>
    <Checkmark />
</motion.div>
```

---

## 9. 安全模型 & 威胁分析

### 9.1 威胁模型 (STRIDE)

```
Spoofing (仿冒):
  ✓ Groth16 证明本身包含身份验证,不可伪造
  ✓ PDA 种子保证 Verifier 程序唯一性
  ✓ 用户必须用自己的私钥签名交易

Tampering (篡改):
  ✓ 证明和公开输入绑定,篡改公开输入导致验证失败
  ✓ 承诺树根存储在 PDA,不可篡改
  ✓ Leaf versioning 防止重放攻击

Repudiation (否认):
  ✓ 选择性披露事件写入 Solana 日志,不可否认
  ✓ Nullifier 公开上链,消耗行为可追溯

Information Disclosure (信息泄露):
  ✓ ZK 保证私有输入不泄露
  ✓ Poseidon 哈希在电路中友好,无侧信道泄露
  ✓ 选择性披露最小化:只公开用户允诺的属性
  ⚠ 风险: 交易频次分析 → 见缓解措施

Denial of Service (拒绝服务):
  ⚠ 风险: 大量无效证明恶意提交
  ✓ 缓解: 提交证明需要支付 CU 成本,经济防御
  ⚠ 风险: 承诺树膨胀
  ✓ 缓解: 过期叶子定期清理,激励清理者

Elevation of Privilege (提权):
  ✓ 撤销操作需要叶子持有者的签名
  ✓ BatchVerify 只验证现有账户,不允许越权
  ✓ Verifier 和 PSTree 分离权限
```

### 9.2 密码学假设 & 安全边界

```
假设:
  1. Poseidon 哈希抗碰撞 (≈128 bit 安全性)
  2. Groth16 知识可靠性假设 (Knowledge Soundness)
  3. 椭圆曲线 BN254 上 DLP 困难
  4. Solana 共识层安全 (≥2/3 honest)

边界:
  客户端 → 可信执行环境 (TEE) 可进一步提升安全性 (未来)
  证明生成 → 浏览器 WASM (侧信道攻击风险低, 因为不涉及长期密钥)

前向量子安全:
  当前方案不抗量子攻击 (Groth16/Plonk 基于椭圆曲线)
  未来升级: FRI-based STARK 可提供抗量子性
```

### 9.3 交易频次分析的缓解

```
问题: 虽然转账金额不可见, 但转账频率 + 公开输入模式可能泄露信息

缓解措施:
  1. 批处理: 单笔交易聚合多个操作,混淆频次模式
  2. 虚设交易 (dummy): 随机插入零价值转账
  3. 时间扰动: 交易延迟随机 0-30 秒
  4. 聚合器: 用户通过代理提交交易,代理聚合后上链
    (类似 MEV 保护 RPC 的架构)
```

---

## 10. 部署架构

### 10.1 Devnet 部署

```
Solana Devnet
├── Verifier Program (PDA)
│   └── 8 accounts for verification keys per proof type
├── PSTree Account (PDA)
│   └── 单个全局树 + 按 namespace 分割
├── Demo: Poker Game Program
│   └── 发牌公平性用 Verifier 验证
├── Demo: Dark Pool Program
│   └── 订单匹配用 Verifier 验证
└── Demo: Voting Program
    └── 投票计数用 Verifier 验证

前端: Vercel (https://privy-svm.vercel.app)
RPC: Helius / QuickNode (高可靠 RPC)
IPFS: 证明元数据存储 (可选)
```

### 10.2 部署脚本

```bash
#!/bin/bash
# deploy-devnet.sh

# 1. 构建所有程序
privy build --release

# 2. 部署 Verifier Program
solana program deploy \
    --url devnet \
    --keypair ~/.config/solana/devnet.json \
    target/deploy/privy_verifier.so

# 3. 初始化 PSTree
privy init-tree --namespace default --url devnet

# 4. 部署 Demo 程序
solana program deploy target/deploy/poker_demo.so --url devnet
solana program deploy target/deploy/darkpool_demo.so --url devnet
solana program deploy target/deploy/voting_demo.so --url devnet

# 5. 验证部署
privy verify-deployment --url devnet

# 6. 前端部署
cd frontend && vercel deploy --prod
```

---

## 11. Demo 场景技术方案

### 11.1 ZK 扑克 Demo

```
流程:
  1. 发牌者程序用可验证随机函数 (VRF) 生成随机种子
  2. 每个玩家对种子 + 自己的非对称密钥做 ZK 证明 → 得到自己的手牌
  3. 玩家下注/弃牌 → 用隐私证明证明下注金额
  4. 回合结束 → 摊牌阶段,玩家选择性披露手牌
  5. Verifier 验证 "玩家在摊牌时真的展示了之前承诺的手牌"

技术亮点:
  - 手牌承诺 → 玩家承诺了自己的手牌,但不能偷换
  - ZK 范围证明 → 证明下注金额在合法范围内 (不暴露具体金额)
  - 选择性披露 → 摊牌时只暴露需要的手牌,其他信息仍隐藏
```

### 11.2 隐私暗池 Demo

```
流程:
  1. 用户提交加密订单到承诺树
     commitment = PoseidonHash(side, pair, amount, price, nullifier)
  2. 撮合引擎读取承诺树,在链下做 ZK 证明: "订单 A 与订单 B 匹配"
  3. 链上 Verifier 验证匹配证明
  4. 如果匹配: PSTree.insert(新仓位承诺), PSTree.consume(旧订单 nullifier)
  5. 订单完成,公开信息只有: "发生了交易" + 手续费

技术亮点:
  - 零信息泄露: 交易对、数量、价格全程不可见
  - 防 front-run: 订单在暗池内隐藏,直到被撮合
  - 金融合规: 手续费公开,可审计
```

### 11.3 隐私投票 Demo

```
流程:
  1. 用户生成 ZK 证明: "我有投票权" (证明持币而不暴露地址)
  2. 用户生成 ZK 证明: "我的投票是 选项A/选项B/选项C"
  3. 链上 Verifier 批验证所有投票
  4. 结果公布: "选项A: 120票, 选项B: 85票, 选项C: 45票"
  5. 隐私保证: 无法追溯谁投了什么

技术亮点:
  - 一票一权: 用 nullifier 防重复投票
  - 抗胁迫: 用户可以事后否认自己投了什么 (因为无法追溯)
  - 公开可验证: 任何人可以独立验证计票结果正确
```

---

## 附录 A: 程序账户结构

```
Verifier Program (PDA, seeds = [b"privy-verifier"])
├── version: u64
├── admin: Pubkey
├── verification_keys: Vec<VerificationKey>
│   ├── VK for Poker Circuit
│   ├── VK for DarkPool Circuit
│   ├── VK for Voting Circuit
│   └── ...
└── total_verifications: u64 (统计)

PSTree Account (PDA, seeds = [b"pstree", namespace])
├── root: [u8; 32]
├── leaf_count: u64
├── namespace: u64
├── nullifier_registry_root: [u8; 32]
└── expired_leaf_count: u64

Proof Submitted Account (临时账号,由客户端创建)
├── proof_data: Vec<u8> (Groth16 证明, <256 bytes)
├── public_inputs: Vec<[u8; 32]> (≤10 项)
└── author: Pubkey (交易发送者)
```

## 附录 B: 关键依赖

```toml
# Cargo.toml (Verifier Program)
[dependencies]
solana-program = "=1.18.0"
ark-bn254 = "0.4"            # BN254 椭圆曲线
ark-groth16 = "0.4"          # Groth16 证明系统
ark-ec = "0.4"               # 椭圆曲线运算
poseidon-rs = "0.2"          # Poseidon 哈希
light-poseidon = "0.2"       # 轻量 Poseidon (Solana 优化)
borsh = "1.5"                # 序列化

# Cargo.toml (Client WASM)
[dependencies]
wasm-bindgen = "0.2"
ark-bn254 = { version = "0.4", features = ["wasm"] }
ark-groth16 = { version = "0.4", features = ["wasm"] }
getrandom = { version = "0.2", features = ["js"] }
web-sys = "0.3"
console_error_panic_hook = "0.1"
```

```json
// package.json (Frontend)
{
  "dependencies": {
    "next": "14.x",
    "react": "19.x",
    "@solana/web3.js": "^1.90.0",
    "@solana/wallet-adapter-react": "^0.15.0",
    "@privy-svm/client": "^0.1.0",
    "framer-motion": "^11.0.0",
    "zustand": "^4.0.0",
    "tailwindcss": "^4.0.0",
    "@shadcn/ui": "latest"
  }
}
```
