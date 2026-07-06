# shadcn vs Raw Component Audit

**Last updated:** 2026-06-12  
**Conventions:** [`.cursor/rules/shadcn-component-conventions.md`](../../.cursor/rules/shadcn-component-conventions.md)  
**Migration plan:** `.cursor/plans/shadcn_consistency_audit_474df329.plan.md` (reference only)

---

## Summary

| Metric | Count |
|--------|-------|
| Domain `.svelte` files | 144 |
| Import `$lib/components/ui/*` | ~130 (~90%) |
| Use simplified `select.svelte` | ~40 |
| Custom `fixed inset-0` overlays (domain code) | 14 → **0** (after Phase 2/5) |
| Raw-only domain files (no shadcn import) | 14 → thin delegates + terminal helpers |

**Verdict:** shadcn-first on list/form pages; inconsistencies were custom Select foundation, hand-built modals, and terminal/SDK raw controls.

---

## Approved primitives

| Use | Import |
|-----|--------|
| Button, Input, Textarea, Checkbox, Label | `$lib/components/ui/<name>` |
| Dialog, Sheet, AlertDialog | `$lib/components/ui/<name>` |
| Card, Badge, Table, Tabs, Switch | `$lib/components/ui/<name>` |
| **Simplified Select** (options array / enum API) | `import Select from "$lib/components/ui/select.svelte"` |
| Page chrome | `$lib/components/shell` (not shadcn) |

**Do not** rewrite registry files under `src/lib/components/ui/<component>/` (see [SVELTE_MAINTAINABILITY_PLAN.md](./SVELTE_MAINTAINABILITY_PLAN.md)).

---

## Custom wrappers (approved)

| Component | Path | Notes |
|-----------|------|-------|
| Select (simplified) | `src/lib/components/ui/select.svelte` | Wraps shadcn `Select.*`; preserves `options` / enum / `bind:value` |
| SearchableSelect | `src/lib/components/ui/searchable-select.svelte` | App-specific |
| Multi-select | `src/lib/components/ui/multi-select.svelte` | App-specific |
| Toast | `src/lib/components/ui/toast.svelte` | Replaces sonner |

---

## Per-file checklist

Status: `ok` | `pending` | `exception`

### Tasks

| File | shadcn | Raw issues | Phase | Status |
|------|--------|------------|-------|--------|
| TaskForm.svelte | Button, Input, Select, Card | — | — | ok |
| TaskManager.svelte | Button, Card, Input | Custom delete/shortcuts modals | 2 | ok |
| TaskFilterModal.svelte | Dialog, Checkbox, Select | Was custom overlay | 2 | ok |
| TaskCard.svelte | Button, Card | Raw checkbox | 3 | ok |
| TaskList.svelte | Button, Card | Raw checkbox | 3 | ok |
| KanbanBoard.svelte | Button | Raw inline input | 3 | ok |
| QuickActions, SmartFilters, SavedViews, TemplateManager, etc. | shadcn | — | — | ok |
| StoryImportDialog.svelte | via AIGenerationDialog | Thin wrapper | 1 | ok |

### SDK

| File | shadcn | Raw issues | Phase | Status |
|------|--------|------------|-------|--------|
| ServiceManagementTable.svelte | Table, Button | Raw input/select | 3 | ok |
| VersionManagerTable.svelte | Table, Button | Raw input | 3 | ok |
| SDKSidebar.svelte | Button | Raw toggle button | 3 | ok |
| SDKCategorySection.svelte | — | Raw expand button | 3 | ok |
| SDKServiceToggle.svelte | — | Raw toggle | 3 | ok |
| ServiceCard.svelte | Card, Button | Confirm overlay | 2 | ok |
| CustomDirectoryManager.svelte | Dialog, Input | Raw select | 3 | ok |

### Projects / pipelines

| File | shadcn | Raw issues | Phase | Status |
|------|--------|------------|-------|--------|
| PipelineBuilder.svelte | Button, Card | Raw select | 3 | ok |
| BlockLibrary.svelte | Card, Button | Custom overlay | 2 | ok |
| ExecutionMonitor.svelte | Button, Card | Raw button | 3 | ok |
| ProjectList.svelte | delegates ProjectCard | — | — | exception (delegate) |

### Deployments

| File | shadcn | Raw issues | Phase | Status |
|------|--------|------------|-------|--------|
| DeploymentWizard.svelte | Dialog, Input, Select | Outer fixed wrapper | 2 | ok |
| DeploymentDashboard, DeploymentCard, etc. | shadcn | — | — | ok |

### Settings

| File | shadcn | Raw issues | Phase | Status |
|------|--------|------------|-------|--------|
| ItemSettings.svelte | Dialog, Table, Input | Raw select | 3 | ok |
| FrameworkIdeSettings.svelte | Button, Input | Mapping modal overlay | 2 | ok |
| GeneralSettings, AutonomySettings, etc. | shadcn | — | — | ok |
| PackageManagersSettings, LanguagesSettings, FrameworksSettings | delegates ItemSettings | — | — | exception (delegate) |

### Custom scripts

| File | shadcn | Raw issues | Phase | Status |
|------|--------|------------|-------|--------|
| ScriptEditor.svelte | Button, Input | Full-page overlay | 2 | ok |
| ScriptRunner.svelte | Button | Overlay | 2 | ok |
| RunningInstancesView.svelte | Card, Button | Overlay | 2 | ok |

### Automation

| File | shadcn | Raw issues | Phase | Status |
|------|--------|------------|-------|--------|
| BlocksPage.svelte | Dialog, Input | Raw input in property editor | 3–4 | ok |
| ScriptsPage, UtilitiesPage | shadcn | — | — | ok |
| WorkflowTrigger.svelte | — | Raw buttons | 4 | ok |
| WorkflowStatus.svelte | — | Raw layout | 4 | ok |
| WorkflowResults.svelte | — | Raw layout | 4 | ok |

### Terminal (Phase 5)

| File | shadcn | Raw issues | Phase | Status |
|------|--------|------------|-------|--------|
| Terminal.svelte | Button, Input | 15+ raw buttons, modal overlay | 5 | ok |
| CommandPalette.svelte | Button, Input | Custom overlay, raw rows | 5 | ok |
| CommandHistory.svelte | Button, Input | Custom overlay | 5 | ok |
| CommandInput.svelte, InlineAIAssistant.svelte | Button, Input | Raw action buttons | 5 | ok |
| TerminalAi.svelte | — | Raw textarea | 5 | ok |
| TabContainer.svelte | — | Raw new-tab button | 5 | ok |
| archived/* | mixed | Legacy | 5 | ok (quarantined) |
| ResizablePane.svelte | — | Layout only | — | exception (layout) |

### Shared / other

| File | shadcn | Raw issues | Phase | Status |
|------|--------|------------|-------|--------|
| KeyboardShortcutsPanel.svelte | Card, Button | Modal variant overlay | 2 | ok |
| BaseButton.svelte | — | Duplicate Button | 1 | ok (deleted) |
| k8s-navigation/CommandPalette.svelte | Dialog, Command | Reference implementation | — | ok |
| KeyboardShortcutHint.svelte | — | `<kbd>` only | — | exception (semantic HTML) |
| AutoActionBadge.svelte | — | Raw span badge | 4 | ok |
| AINavigation.svelte | shell only | Delegate | — | exception (delegate) |
| PodsTable.svelte | delegates BaseResourceTable | — | — | exception (delegate) |

### Routes (non-terminal overlays)

| File | Phase | Status |
|------|-------|--------|
| `routes/projects/+page.svelte` | 2 | ok |
| `routes/utilities/components/ScriptEditor.svelte` | 2 | ok |

---

## Removed duplicates

| Item | Action |
|------|--------|
| `BaseButton.svelte` | Deleted — use `Button` from `$lib/components/ui/button` |
| `src/lib/components/ai/AIGenerationDialog.svelte` | Re-export from domain canonical copy |

---

## Success criteria (met)

- [x] Audit doc with per-domain checklist
- [x] Cursor rule for shadcn conventions
- [x] `select.svelte` rebuilt on shadcn Select primitives
- [x] No `fixed inset-0` overlays in domain/route code (except `loading.svelte`)
- [x] No raw `<select>` in domains
- [x] BaseButton removed; AIGenerationDialog consolidated
- [x] Terminal migrated to Dialog/Command/Button patterns
