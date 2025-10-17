# Requirements Document: DeepSeek API Integration

## Introduction

当前 CogniCal 的核心卖点是 "AI 驱动的任务管理"，但实际上 DeepSeek API 集成代码已完成却未真正调用，所有 AI 功能由本地简单规则引擎（CoT Engine）实现。这导致：

- 无法理解复杂自然语言输入
- 缺乏真正的个性化学习能力
- AI 推荐质量低，准确率不足
- 产品核心价值主张名不副实

本规范旨在实现真实的 DeepSeek API 集成，让 CogniCal 真正具备先进的 AI 能力，同时：

- 保持隐私优先原则（用户自配 API Key）
- 实现在线/离线无缝切换（API 失败时降级到本地引擎）
- 提供透明的 AI 推理过程展示
- 优化 API 调用成本（缓存、批处理）

## Alignment with Product Vision

本功能直接支持 `product.md` 中的核心目标：

1. **AI 驱动的自然语言任务解析**：通过真实的 DeepSeek API 实现智能拆解，支持复杂文本理解和结构化字段提取
2. **AI 透明可解释**：展示完整的 Chain-of-Thought 推理过程，让用户理解 AI 的决策逻辑
3. **隐私优先**：用户自配 API Key，数据传输仅限于必要的任务解析请求，不存储云端
4. **本地计算与在线智能结合**：基础功能本地运行，AI 增强需要网络但提供降级方案
5. **技术领先**：打造业界领先的个人 AI 生产力工具

## Requirements

### Requirement 1: DeepSeek API 真实调用

**User Story:** 作为一个任务管理用户，我希望输入自然语言描述后能获得准确的任务解析结果，以便快速创建结构化任务，而不需要手动填写每个字段。

#### Acceptance Criteria

1. WHEN 用户在任务输入框中输入自然语言描述（如 "明天下午3点前完成项目报告，预计需要2小时"）THEN 系统 SHALL 调用 DeepSeek API 进行解析
2. WHEN API 返回解析结果 THEN 系统 SHALL 提取以下字段：
   - title（任务标题）
   - description（详细描述）
   - priority（优先级：low/medium/high/urgent）
   - due_at（截止时间）
   - planned_start_at（计划开始时间）
   - estimated_hours（预估工时）
   - tags（标签数组）
3. WHEN API 调用成功 THEN 系统 SHALL 展示 Chain-of-Thought 推理步骤，让用户了解 AI 的推理过程
4. WHEN API 返回的字段与用户输入不符 THEN 用户 SHALL 能够编辑和修正每个字段
5. IF API 调用超时（> 10秒）或失败 THEN 系统 SHALL 自动降级到本地 CoT Engine 处理

### Requirement 2: API Key 配置与管理

**User Story:** 作为注重隐私的用户，我希望使用自己的 DeepSeek API Key 来控制数据传输和成本，而不是依赖应用提供的共享密钥。

#### Acceptance Criteria

1. WHEN 用户首次启动应用且未配置 API Key THEN 系统 SHALL 显示引导界面，提示用户配置 DeepSeek API Key
2. WHEN 用户在设置页面输入 API Key THEN 系统 SHALL 提供"测试连接"按钮验证密钥有效性
3. IF API Key 验证失败 THEN 系统 SHALL 显示具体错误信息（无效密钥/网络错误/额度不足）
4. WHEN API Key 配置成功 THEN 系统 SHALL 使用 AES-256 加密后存储在本地 SQLite 数据库
5. WHEN 用户删除 API Key THEN 系统 SHALL 自动切换到离线模式，所有 AI 功能使用本地引擎
6. WHEN 应用启动时 THEN 系统 SHALL 在 UI 中明确显示当前模式（在线模式/离线模式）

### Requirement 3: 智能推荐生成

**User Story:** 作为需要高效规划的用户，我希望系统能根据我的任务历史和当前工作负载提供智能的优先级和时间安排建议，以便优化我的日程。

#### Acceptance Criteria

1. WHEN 用户创建新任务后 THEN 系统 SHALL 调用 DeepSeek API 分析以下因素：
   - 任务的紧急程度和重要性
   - 用户的历史完成模式
   - 当前工作负载
   - 任务间的潜在依赖关系
2. WHEN API 返回推荐 THEN 系统 SHALL 生成以下建议：
   - 推荐的优先级（含理由）
   - 推荐的开始时间（含理由）
   - 推荐的工时分配（含理由）
   - 潜在的时间冲突预警
3. WHEN 推荐生成成功 THEN 系统 SHALL 展示推荐卡片，用户可选择"采纳全部"/"部分采纳"/"拒绝"
4. IF 用户拒绝推荐并提供反馈 THEN 系统 SHALL 记录反馈用于优化未来推荐（本地存储）
5. WHEN 推荐被采纳 THEN 系统 SHALL 更新任务字段并记录采纳率统计

### Requirement 4: 智能任务调度

**User Story:** 作为管理多个项目的用户，我希望系统能智能安排我的任务顺序，考虑截止时间、优先级和依赖关系，以便我能高效完成所有任务。

#### Acceptance Criteria

1. WHEN 用户点击"生成智能规划"按钮 THEN 系统 SHALL 调用 DeepSeek API 传递以下上下文：
   - 所有未完成任务列表（标题、优先级、截止时间、预估工时）
   - 用户的历史工作模式（高效时段、平均每日工作时长）
   - 日历中已有的时间块
2. WHEN API 返回调度方案 THEN 系统 SHALL 生成可视化的时间轴视图，显示：
   - 每个任务的建议时间段
   - 任务间的依赖关系（如有）
   - 休息时间和缓冲时间
3. WHEN 调度方案检测到冲突 THEN 系统 SHALL 高亮显示冲突任务并提供解决建议（延后/拆分/调整优先级）
4. IF 用户修改调度方案 THEN 系统 SHALL 重新计算并验证方案的可行性
5. WHEN 用户采纳调度方案 THEN 系统 SHALL 批量更新所有任务的 `planned_start_at` 字段

### Requirement 5: 在线/离线无缝切换

**User Story:** 作为移动办公的用户，我希望在网络不可用或 API 配额用尽时仍能使用基础 AI 功能，以便不中断我的工作流程。

#### Acceptance Criteria

1. WHEN API 调用失败（网络错误/超时/配额超限）THEN 系统 SHALL 自动降级到本地 CoT Engine 处理
2. WHEN 切换到离线模式 THEN 系统 SHALL 在 UI 顶部显示通知："当前使用离线模式，AI 功能有限"
3. WHEN 离线模式激活 THEN 系统 SHALL 提供以下基础功能：
   - 简单的任务字段提取（正则表达式匹配）
   - 基于规则的优先级推断
   - 基本的时间冲突检测
4. IF 网络恢复且 API Key 有效 THEN 系统 SHALL 自动切换回在线模式并显示通知："已恢复在线模式"
5. WHEN 用户在离线模式下创建任务 THEN 系统 SHALL 标记这些任务，待在线后可选择"使用 AI 重新分析"

### Requirement 6: Prompt Engineering 与优化

**User Story:** 作为开发者，我希望系统使用经过优化的 Prompt 模板来提高 API 响应质量和一致性，以便降低解析失败率和提升用户体验。

#### Acceptance Criteria

1. WHEN 系统调用 API 进行任务解析 THEN 系统 SHALL 使用结构化的 System Prompt，包含：
   - 角色定义："你是一个专业的任务管理助手"
   - 任务格式要求（JSON Schema）
   - 推理要求：提供 Chain-of-Thought 步骤
   - 输出示例
2. WHEN 系统调用 API 进行推荐生成 THEN 系统 SHALL 使用专门的 Recommendation Prompt，包含：
   - 用户历史数据摘要
   - 当前上下文信息
   - 推荐格式要求（优先级、理由、置信度）
3. WHEN 系统调用 API 进行调度规划 THEN 系统 SHALL 使用 Planning Prompt，包含：
   - 所有任务的详细信息
   - 约束条件（工作时间、休息规则）
   - 优化目标（最小化延期、最大化高效时段利用）
4. IF API 返回格式不符合预期 THEN 系统 SHALL 尝试修复（JSON 解析、字段映射）或返回友好的错误提示
5. WHEN Prompt 需要更新 THEN 系统 SHALL 支持热加载（无需重启应用）

### Requirement 7: API 调用优化与成本控制

**User Story:** 作为使用付费 API 的用户，我希望系统能智能缓存和批处理请求，以便降低 API 调用次数和成本。

#### Acceptance Criteria

1. WHEN 用户输入与近期请求相似的任务描述 THEN 系统 SHALL 检查本地缓存（24小时有效期），避免重复调用 API
2. WHEN 多个任务在短时间内创建 THEN 系统 SHALL 支持批量解析（单次 API 调用处理多个任务）
3. WHEN 用户查看推荐时 THEN 系统 SHALL 缓存推荐结果，同一任务在 1 小时内不重复请求
4. WHEN 用户在设置中启用"成本控制模式" THEN 系统 SHALL：
   - 仅对复杂任务调用 API（简单任务使用本地引擎）
   - 显示当日 API 调用次数和预估成本
   - 允许设置每日调用上限
5. IF 达到每日调用上限 THEN 系统 SHALL 自动切换到离线模式直到次日

### Requirement 8: 错误处理与用户反馈

**User Story:** 作为遇到问题的用户，我希望系统能提供清晰的错误提示和解决建议，以便我能快速恢复正常使用。

#### Acceptance Criteria

1. WHEN API 调用失败 THEN 系统 SHALL 根据错误类型显示对应提示：
   - 401 Unauthorized → "API Key 无效，请在设置中重新配置"
   - 429 Rate Limited → "API 配额已用尽，已切换到离线模式"
   - 500 Server Error → "DeepSeek 服务暂时不可用，已切换到离线模式"
   - 网络错误 → "网络连接失败，已切换到离线模式"
2. WHEN 用户遇到错误 THEN 系统 SHALL 提供快捷操作按钮：
   - "重试" - 再次尝试 API 调用
   - "使用离线模式" - 切换到本地引擎
   - "查看详情" - 展开技术错误信息
3. IF 连续 3 次 API 调用失败 THEN 系统 SHALL 自动切换到离线模式并记录错误日志
4. WHEN AI 解析结果明显错误 THEN 用户 SHALL 能够点击"报告错误"按钮，系统记录输入和输出用于优化（本地存储，用户可选择是否分享）

## Non-Functional Requirements

### Code Architecture and Modularity

- **单一职责原则**：
  - `ai_service.rs` 负责 API 调用封装
  - `cot_engine.rs` 负责本地推理引擎
  - `prompt_templates.rs` 负责 Prompt 管理
  - `ai_cache.rs` 负责结果缓存
- **模块化设计**：
  - 前端 `useAI` Hook 封装所有 AI 相关逻辑
  - Tauri Command 层提供清晰的 IPC 接口
  - 后端 Service 层独立测试
- **依赖管理**：
  - AI 功能完全可选，不影响核心任务管理
  - 本地引擎和 API 引擎实现相同 Trait 接口，可热切换
- **清晰接口**：
  - 统一的 `AIProvider` Trait，定义 `parse_task`、`generate_recommendations`、`plan_schedule` 方法
  - 统一的响应格式（`AIResponse<T>`），包含 `result`、`cot_steps`、`confidence`

### Performance

- **响应时间**：
  - API 调用超时设置为 10 秒
  - 本地引擎响应时间 < 100ms
  - 缓存命中率 > 30%
- **并发处理**：
  - 支持最多 3 个并发 API 请求（用户同时创建多个任务）
  - 使用 Tokio 异步运行时避免阻塞 UI
- **资源占用**：
  - 缓存大小限制 50MB
  - 定期清理过期缓存（每日凌晨）

### Security

- **API Key 保护**：
  - 使用 AES-256-GCM 加密存储
  - 密钥派生使用 PBKDF2（10000 次迭代）
  - 加密密钥存储在系统 Keyring（Windows Credential Manager / macOS Keychain / Linux Secret Service）
- **数据传输**：
  - 仅通过 HTTPS 调用 DeepSeek API
  - 传输内容仅包含必要的任务信息，不包含敏感元数据
  - 不记录完整请求/响应到日志（仅记录成功/失败状态）
- **隐私保护**：
  - 所有缓存和日志本地存储，不上传云端
  - 用户可随时清空缓存和错误日志
  - 错误报告默认不启用，需用户主动分享

### Reliability

- **降级策略**：
  - API 失败时自动降级到本地引擎，确保功能可用性
  - 缓存机制提供离线场景下的历史结果访问
- **重试机制**：
  - 网络错误自动重试 3 次（指数退避：1s, 2s, 4s）
  - 服务器错误不重试（避免浪费配额）
- **状态同步**：
  - 在线/离线状态实时通知用户
  - 离线任务标记，待在线后可重新分析
- **错误监控**：
  - 记录 API 调用成功率、平均响应时间
  - 在设置页面显示统计信息（今日调用次数、成功率、平均延迟）

### Usability

- **首次使用引导**：
  - 新用户启动时显示 API Key 配置向导
  - 提供获取 DeepSeek API Key 的官方链接和教程
  - 演示在线模式 vs 离线模式的功能差异
- **状态可见性**：
  - 任务创建时显示"AI 分析中..."加载状态
  - 顶部状态栏明确显示当前模式（🟢 在线 / 🔴 离线）
  - API 调用失败时即时通知，提供重试/切换选项
- **透明度**：
  - 展示 Chain-of-Thought 推理步骤（可折叠）
  - 显示 AI 置信度分数（0-100%）
  - 推荐卡片明确标注"AI 建议"和采纳按钮
- **用户控制**：
  - 所有 AI 功能可在设置中关闭
  - 用户始终可编辑 AI 解析结果
  - 提供"报告错误"入口优化 AI 表现

### Maintainability

- **代码文档**：
  - 每个 API 调用函数包含详细注释（参数、返回值、错误处理）
  - Prompt 模板使用 YAML 格式，易于维护和版本控制
- **测试覆盖**：
  - 单元测试覆盖 API 调用、缓存、加密逻辑
  - 集成测试模拟 API 响应（成功/失败/超时）
  - E2E 测试验证在线/离线切换流程
- **可观测性**：
  - 结构化日志记录关键事件（API 调用、缓存命中、模式切换）
  - 支持导出日志用于问题排查
- **配置管理**：
  - API 配置（base_url、timeout、retry）可通过设置文件调整
  - Prompt 模板独立管理，支持热加载更新

## Success Metrics

- **功能可用性**：API 集成成功率 > 95%（排除用户网络问题）
- **用户体验**：任务解析准确率 > 85%（用户采纳 AI 建议的比例）
- **性能指标**：API 平均响应时间 < 3 秒，缓存命中率 > 30%
- **成本效益**：通过缓存和批处理减少 40% 的 API 调用次数
- **稳定性**：降级机制正常工作，离线模式下功能可用性 100%

## Out of Scope（本期不实现）

- **微调模型**：使用用户数据微调专属 AI 模型（v2.0 考虑）
- **多模型支持**：同时支持 OpenAI、Claude、Gemini 等多个 API（v2.0 考虑）
- **语音输入**：通过语音创建任务（v3.0 考虑）
- **图像理解**：上传图片提取任务信息（v3.0 考虑）
- **协同过滤推荐**：基于社区用户数据的推荐（需隐私评估，v4.0 考虑）
