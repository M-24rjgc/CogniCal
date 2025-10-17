# Project Structure

## Repository Overview

```
CogniCal/
├── src-tauri/               # Tauri Rust backend
│   ├── src/
│   │   ├── main.rs         # Application entry point
│   │   ├── commands/       # Tauri command handlers
│   │   │   ├── mod.rs
│   │   │   ├── task.rs     # Task CRUD commands
│   │   │   ├── calendar.rs # Calendar operations
│   │   │   ├── ai.rs       # AI integration commands
│   │   │   ├── analytics.rs # Analytics commands
│   │   │   └── habit.rs    # Habit tracking commands
│   │   ├── services/       # Business logic layer
│   │   │   ├── mod.rs
│   │   │   ├── task_service.rs
│   │   │   ├── calendar_service.rs
│   │   │   ├── ai_service.rs
│   │   │   ├── cot_engine.rs      # CoT reasoning engine
│   │   │   ├── mcts_solver.rs     # MCTS task decomposition
│   │   │   ├── analytics_service.rs
│   │   │   └── habit_service.rs
│   │   ├── agents/         # Dual-agent system
│   │   │   ├── mod.rs
│   │   │   ├── task_planning_agent.rs
│   │   │   └── life_assistant_agent.rs
│   │   ├── db/             # Database layer
│   │   │   ├── mod.rs
│   │   │   ├── connection.rs      # SQLite connection pool
│   │   │   ├── migrations.rs      # Schema migrations
│   │   │   └── models/            # Database models
│   │   │       ├── mod.rs
│   │   │       ├── task.rs
│   │   │       ├── calendar.rs
│   │   │       ├── habit.rs
│   │   │       └── analytics.rs
│   │   ├── utils/          # Utility functions
│   │   │   ├── mod.rs
│   │   │   ├── datetime.rs
│   │   │   ├── validation.rs
│   │   │   └── cache.rs
│   │   └── error.rs        # Custom error types
│   ├── Cargo.toml          # Rust dependencies
│   ├── tauri.conf.json     # Tauri configuration
│   └── icons/              # Application icons
├── src/                     # React frontend
│   ├── main.tsx            # Application entry point
│   ├── App.tsx             # Root component
│   ├── components/         # React components
│   │   ├── common/         # Shared components
│   │   │   ├── Button.tsx
│   │   │   ├── Input.tsx
│   │   │   ├── Modal.tsx
│   │   │   ├── Loading.tsx
│   │   │   └── ErrorBoundary.tsx
│   │   ├── layout/         # Layout components
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Header.tsx
│   │   │   └── MainLayout.tsx
│   │   ├── task/           # Task management
│   │   │   ├── TaskList.tsx
│   │   │   ├── TaskItem.tsx
│   │   │   ├── TaskForm.tsx
│   │   │   ├── TaskDetails.tsx
│   │   │   ├── TaskDecomposition.tsx
│   │   │   └── DependencyGraph.tsx
│   │   ├── calendar/       # Calendar views
│   │   │   ├── CalendarView.tsx
│   │   │   ├── DayView.tsx
│   │   │   ├── WeekView.tsx
│   │   │   ├── MonthView.tsx
│   │   │   └── TimeBlock.tsx
│   │   ├── analytics/      # Analytics dashboard
│   │   │   ├── Dashboard.tsx
│   │   │   ├── ProductivityChart.tsx
│   │   │   ├── TimeDistribution.tsx
│   │   │   ├── EfficiencyMetrics.tsx
│   │   │   └── TrendAnalysis.tsx
│   │   ├── habit/          # Habit tracking
│   │   │   ├── HabitList.tsx
│   │   │   ├── HabitCard.tsx
│   │   │   ├── HabitForm.tsx
│   │   │   └── StreakDisplay.tsx
│   │   ├── ai/             # AI interaction
│   │   │   ├── ChatInterface.tsx
│   │   │   ├── CotDisplay.tsx
│   │   │   └── RecommendationPanel.tsx
│   │   └── settings/       # Settings
│   │       ├── SettingsPanel.tsx
│   │       ├── ApiKeyManager.tsx
│   │       └── Preferences.tsx
│   ├── pages/              # Page-level components
│   │   ├── Home.tsx
│   │   ├── Tasks.tsx
│   │   ├── Calendar.tsx
│   │   ├── Analytics.tsx
│   │   ├── Habits.tsx
│   │   └── Settings.tsx
│   ├── hooks/              # Custom React hooks
│   │   ├── useTasks.ts
│   │   ├── useCalendar.ts
│   │   ├── useAnalytics.ts
│   │   ├── useHabits.ts
│   │   ├── useAI.ts
│   │   └── useTauriCommands.ts
│   ├── stores/             # Zustand stores
│   │   ├── taskStore.ts
│   │   ├── calendarStore.ts
│   │   ├── uiStore.ts
│   │   └── settingsStore.ts
│   ├── services/           # Frontend services
│   │   ├── tauriApi.ts     # Tauri command wrappers
│   │   └── localCache.ts   # Local storage utilities
│   ├── types/              # TypeScript types
│   │   ├── task.ts
│   │   ├── calendar.ts
│   │   ├── analytics.ts
│   │   ├── habit.ts
│   │   └── ai.ts
│   ├── utils/              # Utility functions
│   │   ├── datetime.ts
│   │   ├── formatters.ts
│   │   ├── validators.ts
│   │   └── constants.ts
│   ├── styles/             # Global styles
│   │   ├── globals.css
│   │   └── tailwind.css
│   └── assets/             # Static assets
│       ├── images/
│       └── icons/
├── public/                  # Public assets
├── tests/                   # Test files
│   ├── unit/               # Unit tests
│   │   ├── rust/
│   │   └── typescript/
│   ├── integration/        # Integration tests
│   └── e2e/                # End-to-end tests
│       └── playwright/
├── docs/                    # Documentation
│   ├── api/                # API documentation
│   ├── architecture/       # Architecture docs
│   └── user-guide/         # User documentation
├── scripts/                 # Build and utility scripts
│   ├── setup.sh
│   ├── build.sh
│   └── release.sh
├── .github/                 # GitHub workflows
│   └── workflows/
│       ├── ci.yml
│       ├── release.yml
│       └── test.yml
├── .spec-workflow/          # Spec workflow documents
│   ├── steering/
│   │   ├── product.md
│   │   ├── tech.md
│   │   └── structure.md
│   └── specs/              # Feature specifications
├── .vscode/                 # VSCode settings
│   ├── settings.json
│   └── extensions.json
├── package.json             # Frontend dependencies
├── pnpm-lock.yaml
├── tsconfig.json            # TypeScript configuration
├── vite.config.ts           # Vite configuration
├── tailwind.config.js       # Tailwind configuration
├── postcss.config.js        # PostCSS configuration
├── .eslintrc.json           # ESLint configuration
├── .prettierrc              # Prettier configuration
├── .gitignore
├── README.md
├── LICENSE
└── CHANGELOG.md
```

## Key Directory Explanations

### `src-tauri/` (Rust Backend)

**Purpose**: Tauri application backend containing all Rust code for business logic, database operations, and system integration.

**Key Components**:

- `commands/`: Tauri command handlers exposed to frontend via IPC
- `services/`: Core business logic and algorithms
- `agents/`: Dual-agent AI system implementation
- `db/`: Database models, migrations, and connection management
- `utils/`: Shared utility functions

**Ownership**: Backend team, Rust developers

---

### `src/` (React Frontend)

**Purpose**: User interface layer built with React, TypeScript, and Tailwind CSS.

**Key Components**:

- `components/`: Reusable UI components organized by feature
- `pages/`: Top-level page components corresponding to routes
- `hooks/`: Custom React hooks for state management and side effects
- `stores/`: Zustand global state stores
- `types/`: TypeScript type definitions
- `services/`: Frontend service layer for API communication

**Ownership**: Frontend team, UI/UX developers

---

### `tests/`

**Purpose**: Comprehensive test suite covering unit, integration, and E2E tests.

**Key Components**:

- `unit/`: Fast, isolated tests for individual functions/components
- `integration/`: Tests for component/service interactions
- `e2e/`: Full user flow tests using Playwright

**Ownership**: QA team, all developers (TDD approach)

---

### `docs/`

**Purpose**: Technical documentation for developers and users.

**Key Components**:

- `api/`: API reference for Tauri commands
- `architecture/`: System design documents
- `user-guide/`: End-user documentation

**Ownership**: Tech writers, senior developers

---

### `.spec-workflow/`

**Purpose**: Specification-driven development workflow documents.

**Key Components**:

- `steering/`: High-level product, tech, and structure documents
- `specs/`: Feature-specific specifications

**Ownership**: Product managers, tech leads

## File Naming Conventions

### Rust Files

- **Snake case**: `task_service.rs`, `cot_engine.rs`
- **Module files**: `mod.rs` for module exports
- **Descriptive names**: Clearly indicate the file's purpose

### TypeScript/React Files

- **PascalCase for components**: `TaskList.tsx`, `CalendarView.tsx`
- **camelCase for utilities**: `useTasks.ts`, `datetime.ts`
- **Type files**: Match the feature name (e.g., `task.ts` for task types)

### Test Files

- **Suffix with `.test` or `.spec`**: `task_service.test.rs`, `TaskList.spec.tsx`
- **Mirror source structure**: Tests in same directory structure as source

## Configuration Files

### `tauri.conf.json`

- **Purpose**: Tauri application configuration
- **Key Settings**:
  - App metadata (name, version, description)
  - Window configuration (size, decorations)
  - Security settings (CSP, allowlist)
  - Build targets and bundles

### `package.json`

- **Purpose**: Frontend dependencies and scripts
- **Key Sections**:
  - Dependencies (React, Tailwind, etc.)
  - Dev dependencies (Vite, TypeScript, etc.)
  - Scripts (dev, build, test)

### `Cargo.toml`

- **Purpose**: Rust dependencies and project metadata
- **Key Sections**:
  - Package metadata
  - Dependencies (Tauri, SQLite, reqwest, etc.)
  - Build configuration

### `tsconfig.json`

- **Purpose**: TypeScript compiler configuration
- **Key Settings**:
  - Target ES version
  - Module resolution strategy
  - Path aliases (@/ for src/)
  - Strict type checking

### `vite.config.ts`

- **Purpose**: Vite build tool configuration
- **Key Settings**:
  - Tauri plugin integration
  - Path aliases
  - Build optimization
  - Development server settings

## Build Artifacts

### Development

```
target/              # Rust build artifacts
  └── debug/         # Debug builds
node_modules/        # Frontend dependencies
dist/                # Vite build output
```

### Production

```
src-tauri/target/
  └── release/       # Optimized Rust builds
      └── bundle/    # Platform-specific installers
          ├── dmg/   # macOS installers
          ├── msi/   # Windows installers
          └── deb/   # Linux packages
```

## Development Workflow

### 1. Feature Development

```
1. Create feature spec in .spec-workflow/specs/
2. Implement Rust services in src-tauri/services/
3. Create Tauri commands in src-tauri/commands/
4. Build React components in src/components/
5. Write tests in tests/
6. Update documentation in docs/
```

### 2. Branch Strategy

```
main                 # Production-ready code
  ├── develop        # Integration branch
  │   ├── feature/*  # Feature branches
  │   ├── bugfix/*   # Bug fix branches
  │   └── refactor/* # Refactoring branches
  └── hotfix/*       # Urgent production fixes
```

### 3. Code Review Process

```
1. Create PR from feature branch to develop
2. Automated CI checks (lint, test, build)
3. Code review by at least 2 developers
4. Address feedback and re-review
5. Merge to develop after approval
6. Deploy to staging for QA testing
```

## Module Dependencies

### Backend Dependencies

```
main.rs
  ├── commands/*      → services/*
  ├── services/*      → db/*, agents/*
  ├── agents/*        → services/*
  └── db/*            → No internal dependencies
```

### Frontend Dependencies

```
App.tsx
  ├── pages/*         → components/*, hooks/*
  ├── components/*    → hooks/*, services/*, types/*
  ├── hooks/*         → services/*, stores/*
  ├── stores/*        → types/*
  └── services/*      → types/*
```

## Data Flow Architecture

### Task Creation Flow

```
User Input (UI)
  ↓
TaskForm.tsx (Component)
  ↓
useTasks.ts (Hook)
  ↓
tauriApi.ts (Service)
  ↓ [Tauri IPC]
commands/task.rs (Command Handler)
  ↓
services/task_service.rs (Business Logic)
  ↓
services/ai_service.rs (AI Integration)
  ↓
db/models/task.rs (Data Access)
  ↓
SQLite Database
  ↓ [Response]
Back to UI (State Update)
```

### Analytics Dashboard Flow

```
User Opens Dashboard (UI)
  ↓
Dashboard.tsx (Component)
  ↓
useAnalytics.ts (Hook)
  ↓
tauriApi.ts (Service)
  ↓ [Tauri IPC]
commands/analytics.rs (Command Handler)
  ↓
services/analytics_service.rs (Business Logic)
  ↓
db/models/analytics.rs (Data Access)
  ↓
SQLite Database
  ↓ [Response]
Render Charts and Metrics (UI)
```

## Environment Configuration

### Development Environment

```env
# .env.development
VITE_API_ENV=development
VITE_LOG_LEVEL=debug
TAURI_ENV=development
```

### Production Environment

```env
# .env.production
VITE_API_ENV=production
VITE_LOG_LEVEL=warn
TAURI_ENV=production
```

### API Keys (Stored in OS Keychain)

- DeepSeek API Key: Managed by Tauri secure storage
- No sensitive data in repository

## Deployment Structure

### Release Artifacts

```
CogniCal-1.0.0/
├── Windows/
│   ├── CogniCal_1.0.0_x64.msi
│   └── CogniCal_1.0.0_x64_en-US.msi.zip
├── macOS/
│   ├── CogniCal_1.0.0_x64.dmg
│   └── CogniCal_1.0.0_aarch64.dmg
├── Linux/
│   ├── CogniCal_1.0.0_amd64.deb
│   └── CogniCal_1.0.0_amd64.AppImage
└── checksums.txt
```

## Documentation Structure

### API Documentation

```
docs/api/
├── tauri-commands.md      # Complete command reference
├── types.md               # TypeScript type definitions
└── error-codes.md         # Error handling guide
```

### Architecture Documentation

```
docs/architecture/
├── system-overview.md     # High-level architecture
├── data-flow.md           # Data flow diagrams
├── ai-integration.md      # AI/CoT implementation details
└── security.md            # Security considerations
```

### User Documentation

```
docs/user-guide/
├── getting-started.md     # Installation and setup
├── features.md            # Feature walkthrough
├── tips-and-tricks.md     # Best practices
└── faq.md                 # Common questions
```

## Scalability Considerations

### Current Phase (MVP)

- Monolithic application structure
- Single-user focus
- Local-only data storage

### Future Phases

- **Plugin System**: `src/plugins/` for extensions
- **Multi-user Support**: `src-tauri/sync/` for P2P sync
- **Web Version**: Separate `src-web/` directory
- **Mobile Apps**: Consider React Native rewrite

## Maintenance & Monitoring

### Log Files Location

- **Development**: Console output
- **Production**:
  - Windows: `%APPDATA%\CogniCal\logs\`
  - macOS: `~/Library/Logs/CogniCal/`
  - Linux: `~/.local/share/CogniCal/logs/`

### Database Location

- **Windows**: `%APPDATA%\CogniCal\data\cognical.db`
- **macOS**: `~/Library/Application Support/CogniCal/cognical.db`
- **Linux**: `~/.local/share/CogniCal/cognical.db`

### Backup Strategy

- Automated daily backups to `{data_dir}/backups/`
- 7-day retention policy
- Manual export to user-specified location
