# Phase 3: Intelligent Analysis & Insights Dashboard - Requirements

## Overview

Phase 3 focuses on building the **Intelligent Analysis & Insights Dashboard** for CogniCal, providing users with comprehensive data-driven insights into their productivity patterns, task completion trends, and work habits. This phase leverages the rich data collected from Phase 2's intelligent planning system to deliver actionable insights and personalized recommendations directly in the dashboard interface.

**Key Insight**: Based on comprehensive project analysis, the dashboard is currently just a placeholder with basic layout but no actual analytics functionality. This phase will transform it into a powerful analytics hub.

## Current State Assessment (2025-10-11 Build)

> 见附件截图（Dashboard / Tasks / Calendar / Settings 页面）。

- Dashboard 仍显示 Phase 1 文案，缺少真实指标、图表与引导，仅提供基础说明。
- Tasks 页存在空状态提示，但尚未串联智能规划、洞察等后续操作；AI 相关筛选仅为占位。
- Calendar 页仅有空白提醒，尚未展示时间块或冲突信息，也缺失与洞察的联动。
- Settings 页完全为空状态，仅提示“即将上线”，无法配置 DeepSeek API、主题、时间偏好等关键项。
- 整体导航仍标注 “Phase 1 - Intelligent Task Parsing”，未体现项目进入 Phase 3 的阶段性成果。

上述现状说明：Phase 3 必须不仅补齐洞察仪表盘，更要替换占位内容、打通任务→规划→洞察闭环，并提供基础配置能力，使应用达到“可用”基线。

## Steering Alignment

- **Value Pillars**: Advances the third核心支柱「洞察反馈与习惯养成」，补足 Phase 1/2 已完成的语义理解与智能调度能力，确保任务→计划→洞察的闭环体验。
- **Target Personas**: 优先满足多项目知识工作者、自我驱动学习者与高自主自由职业者的复盘与自我优化场景，提供个性化、可视化的执行反馈。
- **产品原则遵循**:
  - _隐私优先_：全部分析在本地 SQLite 与 Rust 服务完成，不向外部传输原始数据。
  - _解释透明_：所有 AI 推荐与评分需附带可解释的 CoT 推理摘要与理由说明。
  - _人机协同_：推荐以多方案方式呈现，用户可对洞察提供反馈并驱动后续调度调整。
  - _节奏友好_：洞察卡片与提醒遵循低干扰策略，允许用户自定义节奏与频率。
- **技术路线契合**: 对齐 steering tech.md 中的架构规划——前端通过 Recharts 等可视化库展示指标，后端扩展 `analytics_service` 与 `analytics` 命令，实现 Rust 层指标聚合与缓存。
- **生态扩展性**: 仪表盘组件需模块化设计，为未来插件市场与双智能体协作（生活助手智能体）预留接口，如导出 JSON/Markdown 报告、触发个性化习惯建议。

## Baseline Product Readiness

为确保 Phase 3 交付后即可形成「可用版本」，在洞察仪表盘之外需同步落实以下基础体验：

- **核心流程闭环**：确认任务创建 → 智能规划 → 洞察复盘三段流程均可在桌面端独立完成，且页面间导航与状态同步顺畅。
- **零数据与冷启动**：提供 Dashboard 零数据状态文案、示例数据或快速引导，帮助初次使用者理解指标含义。
- **关键配置入口**：在 Settings 页面开放 DeepSeek API Key、主题、时间偏好等最小必需设置，保障用户能够实际运行 AI 功能。
- **错误与离线兜底**：Analytics 面板在 SQLite 查询失败、离线或未授权时给出清晰提示与重试机制，遵循节奏友好原则不过度打扰。
- **可导出复盘**：支持将核心指标导出为 Markdown/PNG，方便用户阶段复盘或分享成果。
- **发布质检清单**：建立 Phase 3 发布前的 Smoke Checklist（导航、任务 CRUD、规划生成、洞察加载、设置保存、离线打开）并纳入测试用例。
- **界面去占位化**：Dashboard、Tasks、Calendar、Settings 等页面需替换占位文案，实现真实数据展示或交互；全局阶段指示更新为 “Phase 3 - Intelligent Analysis & Insights”。

## User Stories

### Core Dashboard Analytics

**US-3.1: Task Completion Statistics**

- **As a** multi-project knowledge worker
- **I want** to see real-time task completion statistics and trends
- **So that** I can understand my overall productivity patterns and progress

**US-3.2: Time Allocation Analysis**

- **As a** self-directed learner juggling 课程与训练计划
- **I want** to analyze how I spend my time across different task types and priorities
- **So that** I can identify time management optimization opportunities

**US-3.3: Efficiency Pattern Recognition**

- **As a** 高自主的自由职业者
- **I want** to identify my most productive time periods and work patterns
- **So that** I can schedule important tasks during my peak performance hours

**US-3.4: Work Habit Visualization**

- **As a** 节奏敏感的生产力爱好者
- **I want** to visualize my work habits and routines over time with interactive charts
- **So that** I can build and maintain productive work habits

### Advanced Analytics & Insights

**US-3.5: Productivity Scoring System**

- **As a** 希望量化成长的知识工作者
- **I want** to receive a quantifiable productivity score based on multiple metrics
- **So that** I can track my progress and set improvement targets

**US-3.6: Task Priority Recommendation**

- **As a** 需要兼顾客户与自我项目的自由职业者
- **I want** to receive AI-powered task priority recommendations using CoT reasoning
- **So that** I can focus on the most impactful tasks first

**US-3.7: Optimal Work Time Identification**

- **As a** 时间管理精细的学习者
- **I want** to identify my most effective work hours based on historical data
- **So that** I can plan demanding tasks during my peak performance periods

**US-3.8: Task Grouping & Batching Suggestions**

- **As a** 高并行度的产品创业者
- **I want** to receive intelligent task grouping and batching recommendations
- **So that** I can minimize context switching and improve focus

**US-3.9: Workload Prediction**

- **As a** 项目负责人
- **I want** to predict future workload based on current trends and patterns
- **So that** I can prepare for busy periods and avoid burnout

### Personalization & Recommendations

**US-3.10: Personalized Rest Reminders**

- **As a** 需要平衡工作与生活的生活助手用户
- **I want** to receive intelligent rest reminders based on my work intensity and patterns
- **So that** I can maintain sustainable work habits

**US-3.11: Work Rhythm Optimization**

- **As a** 希望养成稳定节奏的学习者
- **I want** to optimize my work-rest cycles based on my natural productivity patterns
- **So that** I can maintain consistent productivity throughout the day

**US-3.12: Performance Trend Analysis**

- **As a** 持续改进的知识型团队成员
- **I want** to track my performance trends over time with detailed metrics
- **So that** I can identify areas for growth and celebrate progress

**US-3.13: Onboarding & Zero-State Guidance**

- **As a** 初次体验 CogniCal 的用户
- **I want** contextual walkthroughs、零数据占位内容与关键指标解释
- **So that** I can在没有历史数据时也理解仪表盘价值并快速上手

**US-3.14: Reliable Core Workflow**

- **As a** 期望立即投入使用的生产力用户
- **I want** 任务→规划→洞察的完整链路在离线或网络不稳定场景下依然可执行，并对异常给出可恢复的指引
- **So that** I can放心将 CogniCal 作为日常主力工具

## Acceptance Criteria

### Dashboard Analytics Requirements

**AC-3.1.1: Task Completion Statistics**

- [ ] Display total tasks completed this week/month with percentage changes
- [ ] Show completion rate percentage with trend indicators
- [ ] Visualize completion trends over time with interactive charts
- [ ] Compare current period with previous period with delta indicators
- [ ] Provide breakdown by task type and priority
- [ ] Allow users to annotate 或收藏关键统计卡片，以便后续复盘

**AC-3.1.2: Time Allocation Analysis**

- [ ] Show time spent by task type (work/study/life/other) with pie charts
- [ ] Display time distribution across priority levels with bar charts
- [ ] Visualize time investment patterns with heat maps
- [ ] Provide time optimization suggestions based on patterns
- [ ] Track estimated vs actual time with variance analysis
- [ ] 允许切换工作日/周末视角以符合节奏友好原则

**AC-3.1.3: Efficiency Metrics**

- [ ] Calculate and display estimated vs actual time ratios with accuracy scores
- [ ] Show on-time completion rates with trend analysis
- [ ] Track task complexity vs completion time correlation
- [ ] Identify efficiency improvement opportunities with specific recommendations
- [ ] Provide performance benchmarks and comparisons
- [ ] 每项效率建议需附带数据来源说明与可行操作建议

### Advanced Analytics Requirements

**AC-3.2.1: Pattern Recognition**

- [ ] Identify peak productivity hours using statistical analysis
- [ ] Detect recurring work patterns with pattern matching algorithms
- [ ] Recognize optimal task sequencing based on historical success
- [ ] Suggest habit formation opportunities with actionable steps
- [ ] Provide personalized insights based on individual patterns

**AC-3.2.2: Productivity Scoring**

- [ ] Develop multi-factor productivity scoring algorithm (0-100 scale)
- [ ] Include task completion, timeliness, efficiency, and consistency metrics
- [ ] Provide historical score tracking with trend visualization
- [ ] Offer improvement recommendations with specific actions
- [ ] Compare scores across different time periods

**AC-3.2.3: AI-Powered Recommendations**

- [ ] Generate task priority suggestions using CoT reasoning from Phase 2
- [ ] Provide optimal scheduling recommendations based on productivity patterns
- [ ] Suggest task grouping strategies to minimize context switching
- [ ] Offer workload balancing advice with capacity planning
- [ ] Integrate with existing planning service for consistency
- [ ] 展示 CoT 推理摘要、置信度与至少两套备选方案，支持用户反馈

### Baseline Readiness Requirements

**AC-3.4.1: Onboarding & Zero-State**

- [ ] Dashboard 在无任务/无规划数据时展示友好空状态，提供核心指标说明与「导入示例数据 / 创建首条任务」操作
- [ ] 首次进入 Dashboard 时弹出/可回看导览（不扰民），引导用户完成任务创建、规划生成、洞察查看三个关键步骤
- [ ] Settings 页面需包含 DeepSeek API Key、时间偏好、界面主题最小配置，并在未配置时提示如何启用 AI 功能
- [ ] 应用导航与页眉更新为 “Phase 3 - Intelligent Analysis & Insights”，并在 Dashboard 中概述 Phase 0-3 已完成能力

**AC-3.4.2: Workflow Reliability**

- [ ] 任务列表、规划面板、洞察仪表盘之间导航保持状态同步，支持从洞察卡片直接跳转至相关任务/规划记录
- [ ] 在 SQLite 查询失败、Tauri 命令异常或无网络时，前端展示可恢复提示并允许离线查看最近缓存的数据
- [ ] 提供导出按钮生成 Markdown/PNG 报告（含日期、关键指标、AI 建议），并支持匿名化敏感字段
- [ ] 制定 Phase 3 Smoke Checklist，覆盖导航、任务 CRUD、规划生成、洞察加载、设置保存、离线打开，纳入 CI 测试或手动测试说明
- [ ] Dashboard、Tasks、Calendar、Settings 页面的占位文案全部替换为动态数据或明确可执行操作（例如 AI 来源筛选、规划时间线、设置表单）

### Technical Requirements

**AC-3.3.1: Data Processing**

- [ ] Efficiently process large datasets from SQLite using optimized queries
- [ ] Implement real-time data aggregation with incremental updates
- [ ] Support historical data analysis with efficient caching
- [ ] Ensure data privacy and security with local-only processing
- [ ] Handle data schema evolution gracefully
- [ ] 提供 `analytics_overview_fetch` 等 Tauri 命令桥接 React 仪表盘与 Rust `analytics_service`

**AC-3.3.2: Performance**

- [ ] Dashboard loads within 2 seconds with progressive loading
- [ ] Charts and visualizations render smoothly with 60fps target
- [ ] Real-time updates without performance degradation
- [ ] Efficient memory usage for large datasets with pagination
- [ ] Support offline analysis with cached data
- [ ] 可视化实现优先采用 Recharts，支持导出 PNG/SVG

**AC-3.3.3: Integration**

- [ ] Seamlessly integrate with existing task data from taskStore
- [ ] Leverage Phase 2 planning algorithms and data structures
- [ ] Support future Phase 4 smart agent integration with extensible API
- [ ] Maintain backward compatibility with existing data models
- [ ] Follow existing UI/UX patterns from shadcn/ui components
- [ ] 预留生活助手智能体调用洞察的接口（如 Tauri 事件或共享缓存键）

## Non-Functional Requirements

### Performance

- Dashboard should handle up to 10,000 task records efficiently with sub-second response times
- Real-time updates should not impact application responsiveness (< 100ms latency)
- Data processing should complete within acceptable time limits (< 500ms for common queries)
- Memory usage should remain stable even with large historical datasets
- Chart rendering should be smooth with no visible lag

### Usability

- Analytics should be presented in intuitive, actionable formats with clear visual hierarchy
- Users should be able to understand insights without technical knowledge through plain language
- Visualizations should be clear and informative with appropriate color schemes
- Dashboard should be responsive and work well on different screen sizes
- Users should be able to customize dashboard layout and metrics
- 每条 AI 洞察须附带 CoT 步骤概要与可操作建议，满足解释透明原则

### Data Privacy

- All analysis should occur locally on user's device with no external data transmission
- No personal data should be transmitted to external services including AI APIs
- Users should have control over data retention policies with clear deletion options
- Analytics should respect user privacy settings and data sharing preferences
- All data should be encrypted at rest in SQLite database
- 洞察导出前需提醒用户包含的敏感字段，提供匿名化选项

### Extensibility

- Analytics framework should support future feature additions with modular architecture
- Data models should accommodate new metrics and insights without breaking changes
- API should be designed for easy integration with future modules and smart agents
- Dashboard components should be reusable and configurable
- Analysis algorithms should be pluggable and replaceable
- 模块拆分需遵循 structure.md 提出的 feature-first 目录规划，便于后续插件与 Agent 接入

## Success Metrics

- **Adoption Rate**: 90% of users regularly use the analytics dashboard at least once per week
- **Time Management**: Users report 25% improvement in time management awareness within 30 days
- **Task Completion**: Task completion rates increase by 15% within 30 days of dashboard usage
- **User Satisfaction**: User satisfaction with insights and recommendations exceeds 4.5/5 rating
- **Actionable Insights**: System identifies at least 3 actionable insights per user per week
- **Performance**: Dashboard loads within 2 seconds and responds to interactions within 100ms
- **Data Accuracy**: Analytics provide 95%+ accurate insights based on user validation
- **Feature Usage**: Core analytics features (completion stats, time analysis) used by 80%+ of users
- **North Star Alignment**: 有效 AI 协助任务数较 Phase 2 提升 ≥ 20%，体现洞察对调度与执行的反哺
- **Workflow Reliability**: 关键路径（任务创建 → 规划 → 洞察）成功率达到 95%+，零数据用户 80% 在引导后完成首条任务
- **Configuration Adoption**: ≥70% 用户完成 DeepSeek API Key 配置并成功触发一次 AI 洞察

## Out of Scope

- **External Calendar Integration**: Integration with external calendar services (reserved for Phase 4)
- **Advanced ML Training**: Advanced machine learning model training beyond existing CoT reasoning
- **Social Features**: Social features or team collaboration capabilities
- **Mobile Development**: Mobile application development or responsive mobile-first design
- **Cloud Sync**: Cloud synchronization features or multi-device data sync
- **Real-time Collaboration**: Real-time collaboration or sharing features
- **Advanced AI Agents**: Implementation of additional AI agents beyond existing planning agent
- **External API Integration**: Integration with external productivity APIs or services
- **Habit模块完备化**：structure.md 中的习惯追踪组件属于后续阶段，不在本次范围
