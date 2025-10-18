# Tasks Document

- [x] 1. Create onboarding progress store and utilities
  - File: src/stores/onboardingStore.ts
  - Additional File: src/utils/onboarding.ts
  - Implement a persisted Zustand store managing onboarding flags, step completion, and replay triggers. Expose helper selectors and constants for reuse by UI layers.
  - Purpose: Centralize onboarding state with durable persistence and reusable step metadata.
  - _Leverage: zustand/middleware persist helper, existing KeyboardShortcutContext for overlay coordination_
  - \_Requirements: Requirement 1, Requirement 3, Non-Functional Requirements (Code Architecture and Modularity, Reliability)
  - _Prompt: Implement the task for spec interactive-onboarding, first run spec-workflow-guide to get the workflow guide then implement the task: Role: TypeScript developer focused on state management | Task: Implement a persisted onboarding store in src/stores/onboardingStore.ts and export reusable tour step metadata from src/utils/onboarding.ts, wiring completion/reset helpers in line with requirements 1 and 3 plus the non-functional reliability constraints | Restrictions: Do not mutate other stores, ensure persistence resets gracefully when schema version changes, keep helper exports tree-shakable | \_Leverage: zustand create/persist APIs, existing KeyboardShortcutContext overlay semantics | \_Requirements: Requirement 1, Requirement 3, Non-Functional Reliability | Success: Store exposes typed actions/selectors, persists state across reloads with schema guard, unit tests compile_

- [x] 2. Build Driver.js-based onboarding orchestrator component
  - File: src/components/onboarding/OnboardingOrchestrator.tsx
  - Additional File: src/components/onboarding/useDriverLifecycle.ts (optional hook for lifecycle separation)
  - Mount a headless component that consumes onboarding store state, initializes Driver.js with step definitions, listens for replay events, and updates progress markers.
  - Purpose: Deliver first-launch tour execution with resume/replay support.
  - _Leverage: driver.js dependency, useLocation from react-router-dom, onboardingStore actions_
  - \_Requirements: Requirement 1, Non-Functional Requirements (Performance, Usability)
  - _Prompt: Implement the task for spec interactive-onboarding, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React engineer experienced with guided tour libraries | Task: Create OnboardingOrchestrator that boots Driver.js using metadata from utils/onboarding.ts, syncs with onboardingStore, resumes from saved step, and dispatches custom events for replay, honoring performance/usability constraints | Restrictions: Avoid blocking rendering (lazy-load driver assets), guard against missing anchors, detach listeners on unmount | \_Leverage: driver.js, react-router-dom useLocation, onboardingStore helpers | \_Requirements: Requirement 1, Non-Functional Performance & Usability | Success: Component auto-starts on eligible profiles, supports resume/replay, no console errors when targets missing_

- [x] 3. Add contextual help popover component and instrumentation
  - File: src/components/help/HelpPopover.tsx
  - Additional Files: src/components/help/HelpIconButton.tsx (optional), updates to dashboard/tasks/calendar analytics components to mount popovers
  - Create reusable `?` icon popover with accessible focus trap and integrate into high-density panels per requirement 2, wiring copy from onboarding utilities.
  - Purpose: Provide inline explanations without disrupting workflows.
  - _Leverage: Radix Popover via shadcn/ui patterns, cn utility, existing UI Button variants_
  - \_Requirements: Requirement 2, Non-Functional Requirements (Usability)
  - _Prompt: Implement the task for spec interactive-onboarding, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React accessibility specialist | Task: Build an accessible HelpPopover component and attach it to dashboard/tasks/calendar analytics panels with copy sourced from utils/onboarding.ts, ensuring keyboard support and Escape handling | Restrictions: Do not alter business logic of host components, respect layout spacing, ensure popovers collapse on route change | \_Leverage: Radix Popover primitives, cn helper, existing Button styles | \_Requirements: Requirement 2, Non-Functional Usability | Success: Popovers pass keyboard navigation tests, explanatory copy renders correctly on all target panels_

- [x] 4. Implement centralized help center dialog
  - File: src/components/help/HelpCenterDialog.tsx
  - Additional File: src/components/help/HelpCenterContent.tsx
  - Build a dialog combining onboarding status card, API key guidance (via settingsStore), keyboard shortcut embed, and quick actions (replay tour, open docs). Integrate with RootLayout state and command palette/shortcut triggers.
  - Purpose: Offer consolidated assistance entry point accessible anywhere.
  - _Leverage: shadcn/ui dialog primitives, KeyboardShortcutsHelp component, useSettingsStore selectors, onboardingStore actions_
  - \_Requirements: Requirement 3, Non-Functional Requirements (Code Architecture and Modularity, Security)
  - _Prompt: Implement the task for spec interactive-onboarding, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend developer skilled in modular UI composition | Task: Create HelpCenterDialog that composes onboarding progress, API key CTA, and shortcut reference while exposing replay/reset callbacks, integrating with RootLayout triggers per requirement 3 and modular/security constraints | Restrictions: Keep sensitive API key data masked, avoid redundant state, ensure dialog focus management aligns with Radix expectations | \_Leverage: shadcn Dialog, KeyboardShortcutsHelp, useSettingsStore, onboardingStore | \_Requirements: Requirement 3, Non-Functional Code Architecture & Security | Success: Dialog opens via sidebar entry and Shift+/ shortcut, displays correct API status, buttons fire replay/reset actions_

- [x] 5. Wire RootLayout triggers, command palette, and shortcuts
  - File: src/routes/index.tsx
  - Additional Files: src/components/layout/Sidebar.tsx (new help entry), src/components/keyboard/CommandPalette.tsx (command list extension)
  - Connect onboarding and help hub toggles into RootLayout: auto-launch orchestrator, extend command palette commands, add sidebar nav item, and ensure keyboard shortcuts respect overlay states.
  - Purpose: Make assistance surfaces discoverable via navigation, commands, and shortcuts.
  - _Leverage: existing KeyboardShortcutContext, command palette infrastructure, sidebar configuration, onboardingStore selectors_
  - \_Requirements: Requirement 1, Requirement 3
  - _Prompt: Implement the task for spec interactive-onboarding, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React routing and UX integration engineer | Task: Update RootLayout and related components to initialize onboarding orchestrator, register help hub command palette entries, add sidebar link, and ensure Shift+/ opens help center while respecting overlay gating per requirements 1 and 3 | Restrictions: Maintain existing shortcut behavior, avoid duplicate navigation state, keep new strings localization-ready | \_Leverage: KeyboardShortcutContext, commandPaletteItems, onboardingStore | \_Requirements: Requirement 1, Requirement 3 | Success: Help hub reachable via sidebar/command/shortcut, orchestrator auto-launch conditions verified in integration tests_

- [x] 6. Testing and QA hardening
  - Files: src/**tests**/onboardingStore.test.ts, src/**tests**/HelpCenterDialog.test.tsx, e2e/help-center.e2e.ts (new Playwright spec)
  - Author unit tests for store logic, integration tests for help dialog behavior, and Playwright flows covering first-run tour and contextual help, ensuring non-functional reliability/performance requirements are verifiable.
  - Purpose: Guard against regressions and validate acceptance criteria end-to-end.
  - _Leverage: Vitest + Testing Library setup, existing Playwright configuration, onboarding utilities_
  - \_Requirements: All requirements (1-3), Non-Functional Requirements (Reliability, Usability, Performance)
  - _Prompt: Implement the task for spec interactive-onboarding, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA-focused engineer with expertise in React testing and Playwright | Task: Add unit, integration, and E2E coverage validating onboarding store transitions, help center rendering, and tour/help flows per requirements 1-3 and reliability/usability constraints | Restrictions: Keep tests deterministic (reset storage between runs), avoid brittle selectors, ensure CI-ready duration | \_Leverage: Vitest Testing Library utilities, Playwright config, onboardingStore helpers | \_Requirements: Requirements 1-3, Non-Functional Reliability & Usability | Success: New tests pass locally and in CI, failure cases are meaningful, coverage proves acceptance criteria_
