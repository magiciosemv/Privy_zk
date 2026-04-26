# Findings — Privy SVM

> 研究记录

## 技术调研

### Solana 隐私现状
- Solana 链上所有数据 100% 公开, 无原生隐私层
- 现有方案: Elusiv (仅交易隐私), Light Protocol (ZK基础设施,非隐私VM)
- 竞品 Aztec (ETH L2) 虽然功能强但要学 Noir 语言, 开发者摩擦大

### ZK 方案对比
- Groth16: 最小证明 (128 bytes), 最快验证, 需要可信设置
- Plonk: 不需要可信设置, 证明稍大 (400 bytes), 验证稍慢
- STARK: 不需要可信设置, 证明大 (10KB+), 但可并行化

### Solana BPF 限制
- 程序大小限制: 16KB per account → ZK 验证器必须轻量
- CU 预算: 200K per tx, 优先预算 20K → 只做验证,证明离线生成
- 不支持浮点数 → ZK 电路只用整数运算

### 浏览器 WASM ZK
- arkworks-rs Rust 库可编译到 WASM
- 浏览器环境中 Groth16 证明生成: ~500ms 可行
- 需要 Web Workers 避免阻塞主线程

## 竞品详细对比

### Aztec Protocol
- EVM L2 rollup
- 使用 Noir 语言
- 合约级隐私
- Gas 费高, 吞吐量低

### Elusiv
- Solana 原生
- 交易级隐私 (mixer)
- 不是通用计算隐私

### Light Protocol
- Solana ZK 基础设施
- 提供 ZK 验证和压缩
- 不是隐私VM,需开发者自己搭电路

## 技术决策

### 为什么用 Solana 而非 Ethereum
1. 并行执行优势: 隐私状态更新可并行
2. TPS: Solana >4000 TPS vs ETH ~15 TPS
3. 手续费: Solana 极低, ZK 验证不贵
4. 生态空白: Solana 无隐私层, 蓝海机会
5. Firedancer 客户端将进一步提升性能
