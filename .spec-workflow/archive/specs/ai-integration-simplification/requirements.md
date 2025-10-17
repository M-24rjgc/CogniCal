# Requirements Document: AI Integration Simplification

## Introduction

当前的 AI 任务解析体验因双提供者结构（DeepSeek 在线 + 本地 CoT 引擎）与复杂的人机交互而显得笨重。用户报告指出：

- DeepSeek API 已经能够提供高质量的结构化输出，本地 CoT 降级仅带来低质量结果和逻辑噪音。
- 前端存在过多按钮、模式提示和反馈入口，操作学习成本高。
- 自动降级缺乏透明度，用户无法明确感知当前结果来源，更无法信任 AI。

本特性旨在精简 AI 解析链路，仅保留 DeepSeek 单一提供者，重构提示词与错误反馈机制，并让前端回归“一键解析、自动填充”的轻量体验。

## Alignment with Product Vision

- **AI 透明可解释**：通过移除无价值的本地 CoT 降级，用明确的状态提示展示在线调用结果，让用户清楚知道 AI 的真实来源与可信度。
- **人机协同决策**：一键解析 + 可编辑结果的模式，回归“AI 给建议、用户做决策”的简单交互，减少干扰。
- **隐私优先、本地掌控**：延续 DeepSeek API Key 本地加密存储策略，精简逻辑同时保持零云端依赖。
- **可持续效率**：减少等待与认知负担，让任务录入恢复高效、可靠的体验。

## Requirements

### Requirement 1: 单一 DeepSeek 解析管道

**User Story:** 作为一名需要快速录入任务的知识工作者，我希望 AI 解析结果稳定且可预测，从而敢于把自然语言描述交给系统处理。

#### Acceptance Criteria

1. WHEN 用户点击“AI 解析”按钮 THEN 系统 SHALL 直接调用 DeepSeek API，并在成功后一次性填充解析字段（无需本地降级流程）。
2. IF DeepSeek API Key 未配置 THEN 系统 SHALL 阻止调用并展示指向设置页面的指引提示。
3. WHEN DeepSeek 返回成功响应 THEN 系统 SHALL 写入统一结构（标题、描述、优先级、时间字段、标签等），并保留链路缓存以支持命中优化。
4. WHEN DeepSeek 调用失败（网络/超时/配额） THEN 系统 SHALL 不落地任何本地伪造结果，并返回结构化错误供前端展示。

### Requirement 2: Prompt 与错误反馈优化

**User Story:** 作为依赖 AI 解析的用户，我希望在失败时得到清晰的原因以及可行的下一步操作，从而判断是重试、检查设置还是改用手动录入。

#### Acceptance Criteria

1. WHEN DeepSeek 返回异常 THEN 后端 SHALL 归一化错误码（如 `MISSING_API_KEY`、`HTTP_TIMEOUT`、`RATE_LIMITED`、`INVALID_RESPONSE`）。
2. WHEN Prompt 构造完成后 THEN 系统 SHALL 在日志中仅记录脱敏后的请求元数据（不含任务原文和 Key），并确保响应格式为严格 JSON。
3. WHEN 前端收到错误码 THEN SHALL 映射为用户友好的提示文案，并提供“重试”或“前往设置”操作。
4. WHEN API 响应结构缺失字段 THEN 系统 SHALL 以 `INVALID_RESPONSE` 返回，并携带 `correlationId` 方便开发定位。

### Requirement 3: AI 解析交互极简化

**User Story:** 作为追求效率的用户，我希望 AI 面板只有必要操作，以最快速的方式完成解析并填充表单。

#### Acceptance Criteria

1. WHEN 解析成功 THEN 前端 SHALL 自动展示“已填充 X 个字段”简洁反馈，并隐藏 CoT 步骤、降级徽标等冗余信息。
2. WHEN 解析失败 THEN 前端 SHALL 显示单一的错误卡片，包含问题描述与一个明显的重试入口；不再展示离线/缓存模式切换。
3. WHEN 用户修改解析结果字段 THEN 系统 SHALL 保留用户修改，不受后续重试解析影响，除非用户手动选择覆盖。
4. WHEN 页面首次载入且已配置 API Key THEN 前端 SHALL 仅展示一个解析入口和当前 Key 状态标签，不显示额外模式开关。

## Non-Functional Requirements

### Code Architecture and Modularity

- 项目必须移除未使用的本地 CoT 相关模块（Rust 与前端），避免死码累积。
- AI Service 保持单一职责：配置加载、DeepSeek 调用、缓存读写、错误规约。
- 前端将 AI 解析逻辑封装为轻量 Hook，减少跨组件状态耦合。

### Performance

- DeepSeek 请求超时时间调整为 ≤ 30 秒，并通过指数退避最多重试 2 次。
- 缓存命中后响应时间 ≤ 300ms；缓存存储大小需定期裁剪，避免数据库膨胀。

### Security

- API Key 仍需加密存储，不得在日志或错误上报中泄露。
- 深度脱敏日志，只保留必要的结构信息与 correlationId。

### Reliability

- 所有错误路径必须返回统一的 `AppError` 类型，前端可预测处理。
- 新增 E2E/集成测试覆盖成功、Key 缺失、超时、配额受限等主流程。

### Usability

- AI 面板操作步骤限制在“两步内完成”：输入描述 → 点击解析。
- 错误提示语言采用简洁中文，并明确下一步操作建议。
