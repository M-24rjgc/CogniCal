# 🎯 CogniCal - 智能任务与时间管理

> **版本**: v1.0.1014 | **发布日期**: 2025年10月14日  
> **状态**: ✅ 正式版 (Production Ready)

CogniCal 是一款基于 Tauri、React 和 TypeScript 构建的桌面应用,将任务管理转化为智能生产力助手。通过 AI 驱动的解析、规划和分析,帮助你更高效地管理时间和任务。

---

## ✨ 核心功能

### 🎯 生产力评分系统

- **Composite Score**: 0-100 productivity rating with dimension breakdowns
- **Multi-dimensional Analysis**: Completion rate, consistency, focus time, workload balance
- **Trend Tracking**: Weekly progress monitoring with actionable insights
- **Smart Explanations**: AI-generated context for score changes

### 🤖 AI-Powered Recommendations

- **Multi-Option Planning**: ≥3 plan options with confidence scoring
- **Conflict Detection**: Automatic identification of scheduling conflicts
- **Offline Fallbacks**: Heuristic algorithms when AI is unavailable
- **Preference Learning**: Decision logging for personalized suggestions

### 📊 Workload Forecasting

- **Multi-Horizon Predictions**: 7/14/30-day capacity planning
- **Risk Assessment**: OK/Warning/Critical workload indicators
- **Proactive Alerts**: Early warnings for capacity constraints
- **Confidence Scoring**: Data quality-based forecast reliability

### 🧘 Wellness & Balance

- **Focus Streak Detection**: 90+ minute continuous work alerts
- **Work Streak Monitoring**: 4+ hour session break reminders
- **Exponential Backoff**: Smart nudge frequency adjustment
- **Weekly Summaries**: Compliance rates and rhythm analysis

### 🔒 Privacy-First AI Feedback

- **👍/👎 Sentiment Capture**: Contextual feedback on AI features
- **Automatic Anonymization**: Sensitive data redaction before storage
- **Opt-Out Controls**: Complete privacy control for users
- **Weekly Digests**: Aggregate insights from community feedback

### 🌐 Community Transparency

- **OSS Badges**: Open source licensing and contribution information
- **Plugin Detection**: Automatic identification of extensions
- **Anonymized Exports**: Privacy-protected data sharing
- **Checksum Verification**: Export integrity validation

## 🏗️ Architecture

### Tech Stack

- **Backend**: Rust + Tauri 2.8.5 + SQLite
- **Frontend**: React 18 + TypeScript 5.8.3 + Tailwind CSS
- **State Management**: Zustand + React Query
- **Testing**: Vitest + Playwright

### Key Services

- `ProductivityScoreService` - Multi-dimensional scoring engine
- `RecommendationOrchestrator` - AI + heuristic planning
- `WorkloadForecastService` - Capacity prediction
- `WellnessService` - Proactive balance monitoring
- `FeedbackService` - Privacy-first AI feedback
- `CommunityService` - Transparency and export

## 📦 Installation

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Tauri CLI

### Quick Start

```bash
# Clone repository
git clone <repository-url>
cd cognical

# Install dependencies
pnpm install

# Build and run
pnpm tauri dev
```

### Development

```bash
# Frontend development
pnpm dev

# Backend development
cd src-tauri && cargo watch -x run

# Run tests
pnpm test                    # Frontend tests
cargo test --tests          # Backend tests
pnpm exec playwright test   # E2E tests
```

## 🧪 Testing

### Test Coverage

- **81 Automated Tests** (59 Rust + 22 Frontend)
- **8 E2E Scenarios** covering complete workflows
- **100% Pass Rate** with deterministic behavior

### Quality Assurance

```bash
# Full test suite
pnpm test:all

# Smoke testing
pnpm test:smoke

# Performance validation
pnpm test:performance
```

## 🔧 Configuration

### Key Settings

- **Productivity Scoring**: Enable/disable scoring features
- **AI Recommendations**: Configure DeepSeek API integration
- **Wellness Nudges**: Set quiet hours and sensitivity
- **Privacy Controls**: Manage data collection and sharing

### Environment Variables

```bash
# DeepSeek API (optional)
DEEPSEEK_API_KEY=your_api_key

# Development flags
TAURI_DEV=true
RUST_LOG=info
```

## 📚 Documentation

- [Phase 4 Architecture](./docs/architecture/phase4.md) - System design and data flows
- [Type System Integration](./docs/phase4-type-system-integration.md) - API contracts and interfaces
- [Test Coverage Report](./docs/PHASE4_TEST_COVERAGE.md) - Quality assurance details
- [Smoke Checklist](./docs/SMOKE-CHECKLIST.md) - Release verification steps

## 🤝 Contributing

We welcome community contributions! Please see our:

- [Community Guidelines](./docs/community/guidelines.md)
- [Development Setup](./docs/development/setup.md)
- [Code of Conduct](./docs/community/code-of-conduct.md)

### Plugin Development

CogniCal supports plugin extensions for:

- Custom productivity metrics
- Integration with external tools
- Alternative recommendation algorithms
- Enhanced visualization components

## 📄 License

MIT License - See [LICENSE](./LICENSE) for details.

## 🐛 Support

- **Issues**: [GitHub Issues](https://github.com/your-org/cognical/issues)
- **Discussions**: [Community Forum](https://github.com/your-org/cognical/discussions)
- **Documentation**: [Project Wiki](https://github.com/your-org/cognical/wiki)

---

**Built with ❤️ by the CogniCal Team**  
_Transforming task management into intelligent productivity assistance_
