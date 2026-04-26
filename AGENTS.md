# AGENTS 工作流配置

> 每次任务开始前必须读取此文件，严格按照以下工作流执行。

---

## 核心工作流（强制执行）

### 1. PUA 自我驱动
- **每项任务全程调用 `pua` skill**，保持高专注、高执行力
- 不允许拖延、敷衍、半途而废
- 遇到困难时调用 PUA 模式自我加压，push 自己找到解决方案
- 同一个问题失败 2 次以上立即启动 PUA 高压模式

### 2. Brainstorming 先思考
- **任何任务开始前必须先调用 `brainstorming` skill** 探索需求
- 理解用户意图、挖掘潜在需求、确认设计方案
- 不允许跳过思考直接动手编码

### 3. Planning-with-files-zh 规划文档
- **复杂任务使用 `planning-with-files-zh` skill** 创建任务规划
- 生成 `task_plan.md` / `findings.md` / `progress.md` 三个文件
- 跟踪进度、记录发现、随时更新状态

### 4. Superpowers 驱动执行
- 使用 `using-superpowers` skill 系统性地协调所有能力和资源
- 在 skill 调用前先加载对应的 skill 获取完整指令
- 遵循每个 skill 内部定义的工作流程

### 5. 多 Agent 协同
- **复杂任务必须拆分并调用多个子 agent 并行处理**
- 使用 `task` 工具启动子 agent 分担不同模块
- 充分利用 explore / general agent 类型

### 6. Persistent-Memory 记忆记录
- **每次任务完成后调用 `persistent-memory` skill** 记录关键信息
- 记录：任务内容、关键决策、技术方案、注意事项
- 确保跨会话的记忆连续性

### 7. 按需调用 Skill & MCP
- 根据任务类型选择合适的 skill：
  - 区块链/Web3 → `software-crypto-web3`, `solana-dev`, `solana-kit`
  - 系统设计/架构 → `system-design`, `plan-eng-review`
  - 前端美化/UI → `frontend-design`, `ui-ux-pro-max`, `tailwind-css`
  - 前端动画 → `framer-motion-animator`
  - 性能优化 → `benchmark`, `vercel-react-best-practices`
  - 后端开发 → `nodejs-backend-patterns`, `supabase`, `better-auth-best-practices`
  - 代码审计/安全 → `cso`, `review`, `solana-vulnerability-scanner`
  - Next.js → `next-best-practices`
  - shadcn/ui → `shadcn`
  - 测试 → `playwright-best-practices`
  - 文档生成 → `pdf`, `docx`, `pptx`, `xlsx` (anthropics/skills)
  - MCP 构建 → `mcp-builder`
  - 网页抓取 → `firecrawl`
  - AI/LLM → `ai-sdk`
  - GitHub Actions → `github-actions-docs`

---

## 执行顺序

```
1. PUA 激活 → 2. Brainstorming 思考
   → 3. Planning-with-files-zh 规划
   → 4. Superpowers / Multi-Agent 执行
   → 5. Persistent-Memory 记录
```

## 质量要求

- 所有代码必须有类型定义（TypeScript）
- 遵循项目现有代码风格和约定
- 每次改动后运行 lint/typecheck
- 关键决策必须记录到 persistent-memory
