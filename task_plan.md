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

### 阶段 3: 核心合约开发
- [ ] ZK 电路自动编译器原型
- [ ] Solana Verifier Program 实现
- [ ] 私有状态承诺树实现
- [ ] 证明生成 SDK (WASM)

### 阶段 4: 前端开发
- [ ] Next.js 项目初始化
- [ ] 3 个 Demo 页面开发
- [ ] 证明浏览器 UI
- [ ] 交互与动画

### 阶段 5: 集成测试 & 演示
- [ ] 端到端流程测试
- [ ] Solana Devnet 部署
- [ ] Demo 录制 / 演示脚本

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
