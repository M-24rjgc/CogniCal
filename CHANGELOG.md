# Changelog

All notable changes to CogniCal will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial development release

## [1.0.0] - 2025-10-14

### Phase 4: Productivity Optimization

#### 🎉 Major Features

**Productivity Scoring Engine**

- Composite productivity score (0-100) with multi-dimensional analysis
- Dimension breakdowns: completion rate, consistency, focus time, workload balance
- Trend tracking with AI-generated explanations
- 200ms performance budget with intelligent caching
- Insufficient data handling with graceful degradation

**AI-Powered Recommendations**

- Multi-option plan generation (≥3 options per request)
- Conflict detection and resolution guidance
- 2-second offline fallback with heuristic algorithms
- Decision logging for preference learning
- Confidence scoring and ranking

**Workload Forecasting**

- 7/14/30-day capacity predictions
- Risk level classification (OK/Warning/Critical)
- Nightly job scheduling (00:05 AM)
- Confidence scoring based on data quality
- Proactive alert banners

**Wellness & Balance Monitoring**

- Focus streak detection (90+ minutes)
- Work streak monitoring (4+ hours)
- Exponential backoff for repeated nudges
- Quiet hours respect (user-configurable)
- Weekly wellness summaries

**Privacy-First AI Feedback**

- 👍/👎 sentiment capture with context snapshots
- Automatic data anonymization via redact utility
- Weekly digest generation (≥5 feedback threshold)
- Opt-out enforcement and data purging
- Regenerative call support

**Community Transparency**

- OSS badge and licensing information
- Plugin/module detection
- Anonymized export bundle generation
- SHA-256 checksum verification
- Review-before-share modal

#### 🛠️ Technical Improvements

**Backend Architecture**

- 6 new Rust services with 485+ lines each
- 7 new database tables with idempotent migrations
- 20 new Tauri commands for IPC communication
- Direct invoke pattern for better type safety
- Comprehensive error handling and logging

**Frontend Architecture**

- 26 new React Query hooks for state management
- 8 new UI components with modern design
- TypeScript type system completion
- Responsive design with accessibility support
- Offline-first architecture

**Testing & Quality**

- 81 automated tests (59 Rust + 22 Frontend)
- 8 E2E scenarios covering complete workflows
- 40+ manual smoke test steps
- 100% test pass rate with deterministic behavior
- Performance benchmarking and validation

#### 🔧 Configuration & Settings

**New Settings**

- `ai_feedback_opt_out` - Privacy control for AI feedback
- `wellness_quiet_hours_start/end` - Nudge-free time ranges
- `workload_capacity_threshold` - Custom capacity limits
- `productivity_scoring_enabled` - Score calculation toggle

**Default Values**

- All Phase 4 features enabled by default
- Safe privacy defaults (opt-in for data sharing)
- Conservative nudge thresholds
- Balanced scoring weights

#### 📊 Performance

**Response Times**

- Productivity scoring: <200ms
- Plan generation: <2s (offline fallback)
- Workload forecasting: <100ms
- Wellness checks: <50ms
- Feedback submission: <100ms

**Memory & Storage**

- Efficient SQLite schema with proper indexing
- Intelligent caching with configurable TTL
- Automatic data cleanup and archiving
- Minimal memory footprint for desktop usage

#### 🔒 Security & Privacy

**Data Protection**

- Automatic redaction of sensitive information
- Local processing for wellness calculations
- Explicit user consent for data exports
- Complete data purging capabilities

**Privacy Controls**

- Granular opt-out for individual features
- Anonymous feedback collection
- No personal data in community exports
- Transparent data usage policies

#### 📚 Documentation

**New Documentation**

- [Phase 4 Architecture](./docs/architecture/phase4.md)
- [Type System Integration](./docs/phase4-type-system-integration.md)
- [Test Coverage Report](./docs/PHASE4_TEST_COVERAGE.md)
- [Updated README](./README.md)

**Updated Documentation**

- [Smoke Checklist](./docs/SMOKE-CHECKLIST.md) - Phase 4 verification
- API documentation for all new commands
- Development setup and contribution guidelines

#### 🐛 Bug Fixes & Improvements

**General**

- Fixed TypeScript configuration issues
- Improved error handling and user feedback
- Enhanced offline mode reliability
- Better cache invalidation strategies

**UI/UX**

- Improved loading states and skeleton screens
- Better accessibility support
- Enhanced mobile responsiveness
- Smoother animations and transitions

### Breaking Changes

**Database Schema**

- 7 new tables added (backward-compatible)
- Existing analytics tables extended with Phase 4 columns
- All migrations are idempotent and safe

**API Changes**

- New Tauri commands for Phase 4 features
- Extended type definitions
- Enhanced validation schemas

**Configuration**

- New settings with safe defaults
- Updated environment variable requirements
- Enhanced privacy controls

### Deprecated

- None in this release

### Removed

- None in this release

### Fixed

- TypeScript language service cache issues
- Toast provider API inconsistencies
- Build configuration warnings
- Test flakiness and timing issues

### Security

- Enhanced data anonymization for AI feedback
- Improved input validation and sanitization
- Better error message security (no sensitive data)
- Secure export bundle generation

---

## Previous Releases

### Phase 3: Analytics & Planning

- Advanced analytics dashboard
- Task planning and scheduling
- Export and reporting features

### Phase 2: Core Task Management

- Basic task CRUD operations
- Tagging and categorization
- Search and filtering

### Phase 1: Foundation

- Project setup and architecture
- Basic UI components
- Development tooling

---

## Migration Guide

### From Phase 3 to Phase 4

1. **Database Migration**

   ```bash
   # Automatic migration on first launch
   cargo run --bin migrate
   ```

2. **Configuration Updates**
   - Review new settings in Settings panel
   - Configure DeepSeek API key if using AI features
   - Set preferred quiet hours for wellness nudges

3. **Feature Enablement**
   - All Phase 4 features enabled by default
   - Can be individually disabled via Settings
   - Privacy controls available for sensitive users

### Rollback Procedure

If needed, Phase 4 can be rolled back by:

1. Disabling features via Settings
2. Using database backup from pre-Phase 4
3. Reverting to Phase 3 binary release

---

_This changelog follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format._
