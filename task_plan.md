# Task Plan — Privy SVM

> 创建时间: 2026-04-26

## 目标

为 2026 全球 Web3 黑客松构建 Privy SVM — Solana 可编程隐私虚拟机。

## 阶段

### 阶段 1: 需求分析与 PRD ✅
- [x] 头脑风暴项目方向
- [x] 确定核心创新点
- [x] 撰写完整 PRD
- 产出: `PRD.md`, `AGENTS.md`

### 阶段 2: 技术设计文档 ✅
- [x] ZK 电路架构设计 (PrivyCC)
- [x] Solana Verifier Program 设计
- [x] 承诺树数据结构设计 (PSTree)
- [x] 自适应路由方案设计
- [x] 前端架构设计
- 产出: `DESIGN.md` (11 章节, ~800 行完整技术设计)

### 阶段 3: 核心合约开发 ✅
- [x] Solana Verifier Program 实现 (Groth16 BN254验证,403行)
- [x] 私有状态承诺树实现 (Poseidon SMT, 588行, 12/12测试通过)
- [x] Rust SDK crate (PrivyClient, MerkleTree, 类型系统)
- [x] CLI 工具链 (privy-cli, 7个子命令)
- [x] 全部编译通过, 单元测试通过
- 提交: bd056c3, 22 files, 11479行代码

### 阶段 4: 前端开发
- [ ] Next.js 项目初始化
- [ ] 3 个 Demo 页面开发
- [ ] 证明浏览器 UI
- [ ] 交互与动画

### 阶段 5: 集成测试 & 演示
- [x] 集成测试脚本编写 (2个测试文件)
- [⚠] Solana Devnet 部署 (SBF工具链版本冲突，verifier.so 构建成功但 ELF 过于复杂)
- [⚠] 端到端流程测试 (需修复 SBF build 后执行)
- 产出: `tests/integration.test.ts`, `tests/privy-svm.ts`

### 阶段 6: 文档
- [ ] README.md
- [ ] Cookbook 开发指南
- [ ] API Reference

## 决策记录
| 日期 | 决策 | 原因 |
|------|------|------|
| 2026-04-26 | 选择 ZK 隐私身份方向 | Solana 缺乏隐私层,痛点明确 |
| 2026-04-26 | 采用字段级隐私注解方案 | 零学习成本,与 Aztec 差异化 |
| 2026-04-26 | 项目命名 Privy SVM | 简洁,体现隐私(Privy) + Solana(SVM) |
