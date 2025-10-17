# Requirements Document - Phase 4 Productivity Optimization

## Introduction

Phase 4 transforms CogniCal from an insights dashboard into a proactive productivity co-pilot. Building on the analytics foundations delivered in Phase 3, this phase introduces multi-dimensional productivity scoring, AI-assisted scheduling, forward-looking workload safeguards, sustainable work rhythm guidance, and feedback-driven model refinement. All experiences continue to honour CogniCal’s privacy-first desktop architecture: user data, scores, and analytics remain local, while DeepSeek API calls (network required) power advanced AI recommendations with full transparency.

## Alignment with Product Vision

- **先进的个人生产力工具**：将被动数据展示升级为即时、可执行的指导，帮助多项目知识工作者保持节奏与交付质量。
- **隐私优先与可解释智能**：所有指标与反馈存储在本地 SQLite，通过 Rust 服务计算；涉及 DeepSeek API 的推理链完整展示，确保用户清楚每次建议的来龙去脉。
- **人机协同与节奏友好**：AI 提供多方案建议与休息提醒，用户始终掌控最终决策，并可根据个人节奏调整触发频率。
- **开源共建**：保持完全免费且透明的功能集，将高级生产力特性与社区治理工具开放给所有用户，为后续插件与团队协作能力铺路。

## Requirements

### Requirement 1 — Multi-Dimensional Productivity Scoring

**User Story:** 作为同时推进多个项目的知识工作者，我希望看到一个透明且可解释的生产力评分体系，以便快速识别需要改进的维度。

#### Acceptance Criteria

1. WHEN 用户打开 Analytics 仪表盘 THEN 系统 SHALL 计算并展示 0-100 的综合生产力分数，至少结合任务完成率、准时率、估时准确度、专注执行情况与休息平衡五个维度。
2. IF 用户悬停或点击评分卡片 THEN 系统 SHALL 展示各维度权重、原始指标、CoT 推理摘要以及分数变动说明。
3. WHEN 系统存在 ≥7 天历史数据 THEN 系统 SHALL 呈现按日/周/月切换的评分趋势图，并保留至少 30 天历史。
4. IF 某维度数据不足 THEN 系统 SHALL 将该维度标记为 "Insufficient Data"，排除其对综合评分的影响，并指引用户如何补足数据。

### Requirement 2 — AI-Assisted Task & Time Recommendations

**User Story:** 作为需要合理编排日程的独立工作者，我希望 AI 能基于当前任务与偏好给出多套执行方案，让我快速选择最合适的计划。

#### Acceptance Criteria

1. WHEN 用户点击 "获取 AI 建议" 或系统检测到可用的规划空档 THEN 后端 SHALL 调用 DeepSeek API（网络可用时）并生成 ≥3 套带有时间窗口、优先级排序及 CoT 推理摘要的执行方案。
2. IF DeepSeek API 不可用（网络离线或请求失败） THEN 系统 SHALL 在 2 秒内回退到最近 7 天内的缓存建议，或提供手动规划入口且提示用户稍后重试。
3. WHEN 某方案与现有日程冲突 THEN 系统 SHALL 用显著颜色标注冲突时间段，并至少提供“调整时间”“拆分任务”“替换任务”三种解决选项及其影响说明。
4. WHEN 用户接受、拒绝或调整任一方案 THEN 系统 SHALL 记录用户选择、时间戳与偏好标签，以便后续模型调优与个性化学习。

### Requirement 3 — Workload Forecasting & Capacity Safeguards

**User Story:** 作为负责交付节奏的项目负责人，我希望系统能够提前预测工作负载并提示风险，从而主动调整计划避免过载。

#### Acceptance Criteria

1. WHEN 每日本地分析任务在本地午夜调度运行 THEN 系统 SHALL 计算未来 7/14/30 天的工作负载预测，结合待办任务、计划时间块与历史吞吐率。
2. IF 任一预测区间的预计工时超过用户在设置中定义的容量阈值 THEN 系统 SHALL 在仪表盘和任务页展示风险横幅，标明严重程度与推荐的缓解措施。
3. WHEN 用户展开某条风险详情 THEN 系统 SHALL 列出贡献任务、假设的估时、置信区间以及形成风险的推理摘要。
4. IF 历史数据不足导致预测置信度 < 40% THEN 系统 SHALL 使用启发式估算并明确标注为 "低置信度"，同时提示需要累积更多数据。

### Requirement 4 — Sustainable Focus & Wellness Nudges

**User Story:** 作为追求长期稳定效率的专注型工作者，我希望系统能在我高强度工作时提供合适的休息提醒和节奏建议，避免疲劳。

#### Acceptance Criteria

1. WHEN 专注会话累计时长或连续工作时段超过用户配置的阈值 THEN 系统 SHALL 推送休息提醒，内容至少包含推荐休息时长、理由与可选的短时放松任务。
2. IF 用户设置了专注模式或安静时段 THEN 所有提醒 SHALL 自动延迟至下一可用窗口，并记录被延迟次数。
3. WHEN 用户对提醒执行“完成”“稍后提醒”或“忽略”操作 THEN 系统 SHALL 记录响应并采用指数退避策略调整后续提醒频率。
4. WHEN 用户请求一周健康摘要 THEN 系统 SHALL 展示休息合规度、专注节奏、异常峰值以及可执行的节奏调优建议。

### Requirement 5 — AI Feedback & Continuous Learning Loop

**User Story:** 作为依赖 AI 建议的用户，我希望能快速反馈 AI 的表现，让系统据此持续改进并保持透明度。

#### Acceptance Criteria

1. WHEN 用户查看 AI 建议、评分解释或预测结果 THEN UI SHALL 提供就地的 👍 / 👎 反馈入口，并允许填写可选备注。
2. IF 用户提交负向反馈 THEN 后端 SHALL 保存当次提示词、上下文快照与匿名化元数据到本地反馈仓库，并在同一会话中提供一次“重新生成”选项（网络可用时）。
3. WHEN 某类负向反馈在 7 天内累计达到可配置阈值 THEN 系统 SHALL 在设置页生成周度摘要，说明采取的调整（如调权、降级特性或暂停建议）。
4. IF 用户在隐私设置中选择退出 AI 反馈收集 THEN 系统 SHALL 立即隐藏反馈控件并提供一键清空历史反馈的能力。

### Requirement 6 — Open-Source Transparency & Community Enablement

**User Story:** 作为依赖 CogniCal 的社区维护者，我希望高级功能保持开源透明，并能快捷地将使用体验反馈给社区。

#### Acceptance Criteria

1. WHEN 应用启动 THEN 系统 SHALL 在欢迎或设置面板展示开源许可证、项目仓库、贡献指南与社区沟通渠道，离线状态下仍可访问本地缓存的说明。
2. IF 用户调用高阶功能（如高级导出、批量建议） THEN 系统 SHALL 无条件执行并在结果区域提示“此功能永久开源免费”，确保无潜在付费门槛。
3. WHEN 用户选择“导出社区反馈包” THEN 系统 SHALL 在本地打包匿名化的使用指标、反馈摘要与系统信息，生成适合提交 GitHub Issue 的 Markdown，并允许用户审核后再手动分享。
4. IF 检测到社区插件或自定义模块注册 THEN 系统 SHALL 在仪表盘展示来源、权限说明与可用性，并在模块缺失时优雅降级而非报错。

## Non-Functional Requirements

### Code Architecture and Modularity

- 扩展 `analytics_service`, `recommendation_service`, `forecasting_service`, `feedback_service`, `community_service` 等 Rust 模块，保持命令层与服务层的清晰边界。
- 前端新增 `useProductivityScore`, `useRecommendations`, `useWellness`, `useCommunityExports` 等 hooks，封装业务逻辑并复用现有 Zustand store。
- 所有共享 DTO 与验证逻辑驻留于 `src/types` 与 Zod schema，确保前后端类型一致并易于测试。

### Performance

- 本地评分、预测与提醒计算 SHALL 在 200 ms 内完成；依赖 DeepSeek 的推理请求在 5 秒超时后自动降级至缓存或手动模式。
- 夜间批处理任务 SHALL 在 60 秒内完成并在独立线程/任务中运行，避免阻塞应用启动或前端渲染。

### Security

- DeepSeek API 调用 SHALL 仅发送最小必要的上下文（去除个人备注等敏感信息），并通过 HTTPS 传输。
- 所有评分、反馈、预测与社区导出数据 SHALL 加密或混淆存储在本地安全区域，导出前需用户确认。
- 用户可随时在设置中清除缓存建议、反馈记录与导出包，触发安全擦除流程。

### Reliability

- AI 建议、预测与提醒服务 SHALL 在网络失败、SQLite 写冲突或模型出错时返回可操作的降级状态，不得导致应用崩溃。
- 缓存建议与预测结果 SHALL 标注生成时间并在 7 天后自动失效，以防使用过期数据。
- 社区导出与插件检测流程 SHALL 在异常时写入结构化日志，便于诊断。

### Usability

- 新增的评分、建议、提醒与反馈界面 SHALL 支持键盘导航、屏幕阅读器描述，并自动适配暗/亮主题。
- 所有解释文本保持简洁（≤140 字），并提供“了解更多”链接跳转到详细面板或文档。
- 用户在任何关键操作（接受建议、导出反馈、清除数据）前 SHALL 收到明确的确认提示，以防误操作。
