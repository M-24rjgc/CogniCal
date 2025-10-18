# Requirements Document

## Introduction

Implement an interactive onboarding and help experience that guides first-time CogniCal users through core workflows and keeps contextual assistance one click away. The feature must combine a first-launch walkthrough, inline explanations for complex panels, and a consolidated help surface so newcomers understand what CogniCal does without reading external documentation.

## Alignment with Product Vision

- Reinforces the product principle of **AI 透明可解释** by surfacing why modules exist and how to use them.
- Supports **可持续效率与节奏友好** by providing gentle guidance instead of overwhelming users with dense dashboards.
- Improves **用户粘性** and NPS goals by shortening time-to-value for new knowledge workers and self-driven learners.

## Requirements

### Requirement 1

**User Story:** As a first-time CogniCal user, I want a guided tour that highlights the most important areas of the app, so that I immediately understand where to start and how AI features help me.

#### Acceptance Criteria

1. WHEN the desktop app launches for a profile that has not completed onboarding THEN the system SHALL show an overlay walkthrough covering at least dashboard overview, task creation, AI parsing, and settings/API key panels.
2. IF the user dismisses the tour mid-way THEN the system SHALL record progress and SHALL not re-run completed steps when the user resumes the tour from the help center.
3. WHEN the user finishes all tour steps AND confirms completion THEN the system SHALL persist a flag so the tour does not auto-run on subsequent launches, while keeping a manual restart option.

### Requirement 2

**User Story:** As a returning user exploring a module, I want contextual help icons that explain advanced sections, so that I can recall functionality without leaving my current view.

#### Acceptance Criteria

1. WHEN a user activates a "?" help affordance on dashboard, tasks, calendar, or analytics cards THEN the system SHALL present a concise explanation plus links to deeper resources if available.
2. IF a help popover is open THEN the system SHALL trap focus within the popover for keyboard navigation and SHALL close it on Escape without affecting underlying data.
3. WHEN the theme changes or the layout adjusts responsively THEN the contextual help controls SHALL remain discoverable and accessible via keyboard tab order.

### Requirement 3

**User Story:** As any user needing assistance, I want a centralized help & resources hub, so that I can revisit tutorials, keyboard shortcuts, and troubleshooting tips from one place.

#### Acceptance Criteria

1. WHEN the user opens the help hub via sidebar entry, command palette command, or keyboard shortcut (e.g., Shift + /) THEN the system SHALL surface onboarding status, quick links to replay the tour, keyboard shortcut reference, and API key setup guidance.
2. IF the user has not configured a DeepSeek API key THEN the help hub SHALL highlight the configuration step and SHALL deep-link to the settings panel.
3. WHEN the user views help content offline THEN the system SHALL rely on bundled/local content and SHALL gracefully indicate if external docs are unavailable.

## Non-Functional Requirements

### Code Architecture and Modularity
- Encapsulate onboarding state in a dedicated store or service with clear persistence boundaries (e.g., settingsStore or new onboardingStore).
- Keep UI elements (tour steps, help popovers, hub panel) as reusable components that respect existing layout providers.
- Expose a single integration point for triggering the tour to avoid duplicated logic across modules.

### Performance
- First-launch tour initialization SHALL add no more than 150ms to app startup on mid-tier hardware.
- Contextual help components SHALL lazy-load heavy assets (e.g., illustrations) only when invoked.

### Security
- Do not transmit onboarding interactions to external services; all state remains local.
- Validate that help content with dynamic links only points to whitelisted internal routes or bundled markdown.

### Reliability
- Tour and help surfaces SHALL fail safe: if onboarding data is corrupt or unavailable, the app SHALL continue to function without blocking navigation.
- Persistence logic SHALL withstand abrupt app closures without leaving the user stuck in an unskippable tour.

### Usability
- Provide full keyboard navigation for tour steps, help popovers, and the help hub, meeting WCAG 2.1 AA expectations.
- Support localization readiness by structuring copy in translation-friendly resources.
- Ensure the help hub is reachable within two interactions from any primary screen.
