# Requirements Document

## Introduction

为 CogniCal 的 Phase 1 —— “智能任务解析 + CoT 推理引擎” 提供明确的产品需求，目标是在现有基础任务管理能力之上，引入自然语言任务创建、AI 推理增强字段以及可视化 CoT 思维链，从而显著缩短用户的任务规划时间并提升执行确定性。

## Alignment with Product Vision

该阶段的交付直指产品愿景中的“CoT 推理驱动的智能任务管理”与“智能但透明”的原则：

- 通过 DeepSeek API 和自研 CoT 引擎，让用户以自然语言快速创建结构化任务，减少 30% 的手动录入成本。
- 生成复杂度、建议时间等增强字段，并展示思维链，帮助用户理解 AI 决策，强化信任。
- 为后续智能调度、双智能体协作铺设数据与接口基础，延续 Phase 0 的本地隐私与离线优先策略。

## Requirements

### Requirement 1 — 自然语言任务解析（NL → 结构化字段）

**User Story:** 作为需要快速记录任务的知识工作者，我希望直接用自然语言描述任务，并由系统自动补齐必填字段，以便在忙碌场景下也能高效录入。

#### Acceptance Criteria

1. WHEN 用户在任务创建界面输入自然语言描述 AND 点击“AI 解析”， THEN 系统 SHALL 调用 DeepSeek API 并返回标题、描述、优先级、计划开始时间、截止时间、预估工时、任务类型、标签等结构化字段。
2. IF AI 返回的字段缺失或置信度低 THEN 系统 SHALL 高亮标记缺口并提示用户手动补全，否则不得保存任务。
3. WHEN 解析失败（网络错误或配额不足） THEN 系统 SHALL 提供重试与手动填写选项，同时保留原始输入。

### Requirement 2 — CoT 推理增强字段与可视化

**User Story:** 作为需要评估任务难度与排期的团队负责人，我希望系统给出复杂度评分、AI 建议开始时间、专注模式建议及时间效率预测，并展示 AI 的推理步骤，以便更有信心地安排工作。

#### Acceptance Criteria

1. WHEN 任务解析成功 THEN 系统 SHALL 生成并回填 complexity_score（0-10）、ai_suggested_start_time、focus_mode_recommendation、efficiency_prediction 等增强字段。
2. IF 用户展开“查看 AI 推理” THEN 系统 SHALL 展示链路化的思维步骤（≥4 个步骤，含结论）并支持复制摘要。
3. WHEN 用户对增强字段手动调整 THEN 系统 SHALL 记录用户反馈并在后端存档，用于后续模型调优与缓存命中策略。

### Requirement 3 — 数据持久化与检索扩展

**User Story:** 作为希望复盘与筛选任务的高效管理者，我想根据 AI 字段进行筛选和统计，从而识别高复杂度任务或需要专注模式的任务。

#### Acceptance Criteria

1. WHEN 新字段写入数据库 THEN 系统 SHALL 确保 SQLite Schema 已扩展，字段具备默认值与非空约束，且不会破坏现有任务数据。
2. IF 用户在任务列表中使用过滤器（复杂度、任务类型、标签、AI 建议时间范围） THEN 系统 SHALL 在 ≤200ms 内返回结果（基于本地数据）。
3. WHEN 用户导出任务或触发本地备份 THEN 系统 SHALL 包含所有新增字段，并在导出元数据中注明数据生成时间与 AI 版本。

### Requirement 4 — 推理缓存与成本控制

**User Story:** 作为需要控制 AI 成本的个人用户，我希望系统能够缓存常见任务解析结果，并在 API 失败或离线时回退到最近的有效推理结果。

#### Acceptance Criteria

1. WHEN 用户提交的描述与缓存中的任务语义相似度 ≥ 0.85 THEN 系统 SHALL 优先返回缓存结果并明确标注“来自缓存”。
2. IF DeepSeek API 返回错误 OR 当前处于离线模式 THEN 系统 SHALL 在 1 秒内返回最近一次成功解析结果，并提示用户稍后重试。
3. WHEN 缓存命中率、API 调用次数、平均响应时长 等指标达到监控阈值 THEN 系统 SHALL 记录日志以供 Phase 2 的智能调度模块读取。

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**: 每个 React 组件、Zustand store、Rust 服务模块都应聚焦单一职责，例如 AI 解析服务与缓存管理需拆分独立模块。
- **Modular Design**: 前端应将表单、AI 解析面板、推理展示等拆分为可复用组件；后端将 DeepSeek 客户端、CoT 引擎、缓存层抽象为独立服务。
- **Dependency Management**: 限制前端组件对全局 store 的直接依赖，后端命令层只依赖服务接口不直接操作数据库。
- **Clear Interfaces**: 定义前后端共享的 TypeScript/Rust DTO，并在 tauri 命令层进行严格类型校验。

### Performance

- AI 解析触发后的首次响应时间 ≤ 3 秒，缓存命中时 ≤ 1 秒。
- 前端 CoT 结果面板展开渲染时间 ≤ 100 ms。
- 新增字段的列表筛选、排序操作需控制在 200 ms 内返回结果。

### Security

- DeepSeek API Key 存储在用户本地安全配置中，前端不可直接访问明文 Key。
- 任务与推理缓存数据仅存储在本地 SQLite，导出需经用户确认。
- 所有 AI 响应写入前需通过输入验证与长度限制，防止恶意 payload。

### Reliability

- DeepSeek 调用失败时需保证任务创建流程可降级为手动输入，不得造成数据丢失。
- 推理缓存需具备 7 天 TTL 自动清理策略，并在数据损坏时自动重建。
- Tauri 命令需捕获所有错误并返回结构化 AppError，确保前端可展示友好提示。

### Usability

- 任务创建对话框中的 AI 解析状态需有可视化进度与反馈（加载、成功、失败）。
- 推理结果摘要需提供“复制”与“反馈（有帮助/无帮助）”交互入口。
- 在深色与浅色主题下都需保持可读性，尤其是 CoT 步骤的层级与高亮。
