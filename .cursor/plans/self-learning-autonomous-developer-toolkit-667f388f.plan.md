<!-- 667f388f-50e7-47ab-9bcc-e7792b2b474f 604ff487-55d7-4a38-8dae-b336c047c102 -->
# Self-Learning Autonomous Developer Toolkit Implementation Plan

## Overview

Transform Portal Desktop into an intelligent, self-learning system that autonomously adapts to user workflows, learns from project patterns, code patterns, and best practices across projects. This implementation prioritizes **100% local processing** (no cloud), user-configurable ML intensity for performance tradeoffs, and a lightweight embedded automation system optimized for Tauri applications.

## Architecture Strategy

### Core Principles

1. **Local-First Learning**: All learning happens on-device using lightweight models and pattern recognition (Ollama for advanced inference)
2. **Progressive Autonomy**: System starts with suggestions, learns user preferences, then gradually increases autonomy over time
3. **Performance-Conscious**: User-configurable ML intensity setting balances intelligence vs performance
4. **Privacy-Preserving**: 100% local processing - no data leaves the device ever
5. **Tauri-Optimized Automation**: Embedded workflow engine designed specifically for Tauri (no heavy n8n dependency)
6. **Code Pattern Learning**: Learn from code patterns, best practices, and project structures across all user projects

## Implementation Components

### 1. Learning Domain (Rust Backend)

**Location**: `src-tauri/src/domains/learning/`

**Domain Structure** (Following DDD Pattern - Isolated & Decoupled):

```
domains/learning/
├── entities/
│   ├── mod.rs
│   ├── learned_pattern.rs         # SeaORM entity (Entity, ActiveModel)
│   ├── user_preference.rs          # SeaORM entity
│   └── learning_event.rs          # SeaORM entity
├── repositories/
│   ├── mod.rs
│   ├── learned_pattern_repository.rs   # Repository using SeaORM (NO raw queries)
│   ├── user_preference_repository.rs   # Repository using SeaORM
│   └── learning_event_repository.rs    # Repository using SeaORM
├── services/
│   ├── mod.rs
│   ├── learning_service.rs             # Core orchestrator
│   ├── pattern_matcher.rs              # Pattern recognition algorithms
│   ├── preference_engine.rs            # Preference learning
│   ├── context_analyzer.rs              # Context understanding
│   ├── code_pattern_analyzer.rs         # Code pattern learning (NEW)
│   └── ml_intensity_manager.rs          # ML intensity config (NEW)
├── commands.rs                          # Tauri commands
└── mod.rs                               # Domain exports

migrations/
└── mXXXXXX_create_learning_tables.rs   # SeaORM migration (NO raw SQL)
```

**Key Features**:

- Learn command patterns (frequency, context, sequences)
- Learn project setup preferences (frameworks, package managers, configurations)
- Learn workflow patterns (common task sequences)
- Learn IDE/editor preferences per project type
- **Learn code patterns and best practices across projects** (NEW)
- Pattern matching for predictive suggestions
- **Configurable ML intensity** (Fast/Light/Medium/Deep modes) (NEW)

**Database Schema** (Using SeaORM Migrations - NO raw SQL):

- `learned_patterns` table: id, pattern_type, pattern_data (JSON), context, frequency, last_used, success_rate, created_at
- `user_preferences` table: id, preference_type, context, preference_value (JSON), confidence, learned_from, created_at, updated_at
- `learning_events` table: id, event_type, event_data (JSON), outcome, context, created_at
- All access via SeaORM Entity/ActiveModel through repositories - **NO raw queries**

### 2. Intelligent Suggestions System (TypeScript Frontend)

**Location**: `src/lib/domains/learning/`

**Components**:

- `services/learningService.ts`: Frontend learning service
- `services/suggestionEngine.ts`: Real-time suggestion engine
- `stores/learningStore.ts`: Learning state management
- `components/SuggestionPanel.svelte`: UI for showing suggestions
- `components/AutoActionBadge.svelte`: Visual indicator for auto-actions

**Features**:

- Context-aware suggestions (e.g., "You usually use Python 3.11 for Flask projects")
- Predictive actions (e.g., "Auto-setup Node.js project based on your patterns?")
- Command completion suggestions based on history
- Project setup automation suggestions

### 3. Embedded Workflow Engine (Rust)

**Location**: `src-tauri/src/domains/automation/workflow_engine.rs`

**Purpose**: Replace heavy n8n dependency for common workflows

**Features**:

- Lightweight workflow execution (YAML/JSON-based)
- Conditional logic
- Command execution
- Pattern matching triggers
- Built-in actions: install deps, run commands, create files, configure settings

**Performance Benefits**:

- No HTTP overhead
- No separate process
- Direct database access
- Faster execution

### 4. Autonomous Action System

**Location**: `src-tauri/src/domains/autonomy/`

**Components**:

- `autonomy_service.rs`: Decision engine for autonomous actions
- `action_classifier.rs`: Classifies action safety levels
- `approval_manager.rs`: Manages which actions need approval
- `commands.rs`: Commands for autonomy control

**Safety Levels**:

- **Safe** (auto-execute): Read-only operations, suggestions, UI updates
- **Low Risk** (auto-execute after threshold): Common commands (npm install, git status)
- **Medium Risk** (suggest first): Configuration changes, file creation
- **High Risk** (always confirm): File deletion, system changes, destructive operations

**Learning Progression**:

- Week 1-2: Observation only, generate suggestions
- Week 3-4: Auto-execute safe actions with user feedback
- Month 2+: Increase autonomy based on success rate and user approval patterns

### 5. Pattern Recognition & Context Awareness

**Location**: `src-tauri/src/domains/learning/analyzers/`

**Components**:

- `command_pattern_analyzer.rs`: Analyzes terminal command patterns
- `project_structure_analyzer.rs`: Learns project templates and structures
- `framework_detector.rs`: Enhanced framework detection with learning
- `dependency_analyzer.rs`: Learns dependency patterns

**Algorithms**:

- Simple frequency analysis for patterns
- Sequence pattern matching (Markov-like chains for command sequences)
- Template matching for project structures
- Weighted scoring for context matching

### 6. Extensible Learning Architecture

**Plugin-Style Learning Modules** (Designed for Future Extensions):

The learning system is built with extensibility as a core principle, allowing easy addition of new domains/features:

```
domains/learning/
├── core/                          # Core learning infrastructure
│   ├── learning_engine.rs         # Base learning orchestrator
│   ├── pattern_registry.rs        # Registry for pattern types (extensible)
│   └── adapter_trait.rs           # Trait for domain learning adapters
├── adapters/                      # Domain-specific learning adapters (extensible)
│   ├── mod.rs
│   ├── project_learning_adapter.rs    # Projects domain adapter
│   ├── sdk_learning_adapter.rs        # SDK domain adapter
│   ├── terminal_learning_adapter.rs   # Terminal domain adapter
│   ├── ide_learning_adapter.rs        # IDE domain adapter
│   ├── task_learning_adapter.rs       # Tasks domain adapter
│   └── custom_adapter_template.rs     # Template for new adapters (NEW)
└── patterns/                      # Pattern type definitions (extensible)
    ├── mod.rs
    ├── command_pattern.rs
    ├── workflow_pattern.rs
    ├── code_pattern.rs
    └── custom_pattern_trait.rs     # Trait for new pattern types (NEW)
```

**Extension Points**:

1. **New Pattern Types**: Implement `LearningPattern` trait to add new pattern types
2. **Domain Adapters**: Implement `LearningAdapter` trait to connect new domains
3. **Context Providers**: Implement `ContextProvider` trait to add new context sources
4. **ML Processors**: Implement `MLProcessor` trait to add new ML algorithms

**Integration with Existing & Future Systems**:

- **Current Domains**: 
  - Projects: Learn project setup patterns, suggest configurations
  - SDK: Learn version preferences, auto-select based on project type
  - Terminal: Learn command patterns, suggest completions
  - IDE: Learn IDE preferences per project type
  - Tasks: Learn task creation patterns, suggest templates
  - Automation: Enhance with learned workflows

- **Future Domains** (Easy to Add):
  - **Deployments**: Learn deployment patterns, suggest environments
  - **Credentials**: Learn credential management patterns
  - **Any New Domain**: Just implement `LearningAdapter` trait

**Extension Example** (Adding New Domain Learning):

```rust
// New domain adapter (e.g., for deployments domain)
pub struct DeploymentLearningAdapter;

impl LearningAdapter for DeploymentLearningAdapter {
    fn domain_name(&self) -> &str { "deployments" }
    
    fn collect_patterns(&self, context: &str) -> Vec<Pattern> {
        // Collect deployment-specific patterns
    }
    
    fn generate_suggestions(&self, context: &str) -> Vec<Suggestion> {
        // Generate deployment-specific suggestions
    }
}

// Register adapter (one line)
learning_engine.register_adapter(Box::new(DeploymentLearningAdapter));
```

**Benefits**:

- **Zero Breaking Changes**: New domains don't affect existing learning
- **Isolated**: Each domain adapter is decoupled
- **Composable**: Mix and match learning from different domains
- **Testable**: Each adapter can be tested independently
- **Performance**: Only active adapters consume resources

## Implementation Phases

### Phase 1: Learning Foundation (Week 1-2)

1. Create learning domain structure (Rust backend)
2. Implement database migrations for learning tables
3. Create learning repository for data persistence
4. Implement basic pattern collection (command logging, event tracking)
5. Create frontend learning service integration

### Phase 2: Pattern Recognition (Week 3-4)

1. Implement pattern matcher with frequency analysis
2. Create command pattern analyzer
3. Implement project structure analyzer
4. Build preference engine
5. Add suggestion engine frontend

### Phase 3: Intelligent Suggestions (Week 5-6)

1. Create suggestion UI components
2. Implement contextual suggestion system
3. Add predictive action system
4. Integrate with existing domains (SDK, Projects, Terminal)
5. Add user feedback loop (accept/reject suggestions)

### Phase 4: Embedded Automation (Week 7-8)

1. Create lightweight workflow engine
2. Migrate common n8n workflows to embedded engine
3. Add workflow learning from user actions
4. Implement automatic workflow suggestions
5. Optimize performance (lazy loading, background processing)

### Phase 5: Autonomous Actions (Week 9-10)

1. Implement action safety classifier
2. Create autonomy service
3. Add progressive autonomy system
4. Build approval manager
5. Add autonomy settings UI

### Phase 6: Optimization & Polish (Week 11-12)

1. Performance optimization (database indexes, query optimization)
2. Add learning analytics UI
3. Implement learning reset/export/import
4. Add privacy controls (opt-out, data deletion)
5. Comprehensive testing

## Technical Details

### Performance Optimizations

- **Lazy Learning**: Background processing, only analyze when idle
- **Efficient Storage**: Compressed JSON patterns, periodic cleanup of old patterns
- **Caching**: Cache frequent pattern matches, suggestion results
- **Indexed Queries**: Database indexes on pattern_type, context, frequency
- **Batch Processing**: Learn patterns in batches, not real-time

### Privacy & Security

- All data stored locally in SQLite
- Optional encryption for sensitive learned patterns
- User control: view, delete, export learned data
- No telemetry unless explicitly enabled

### n8n Integration Strategy

- Keep n8n as **optional** enhancement
- Use for complex workflows only (external APIs, complex orchestration)
- Embedded engine handles 80% of common workflows
- Seamless fallback: if n8n unavailable, use embedded engine

### Data Structure Examples

**Learned Pattern Example**:

```json
{
  "pattern_type": "command_sequence",
  "pattern": ["cd project", "npm install", "npm run dev"],
  "context": "nodejs_project",
  "frequency": 45,
  "success_rate": 0.98,
  "last_used": "2025-01-15T10:30:00Z"
}
```

**User Preference Example**:

```json
{
  "preference_type": "sdk_version",
  "context": "python_flask_project",
  "preference_value": "python:3.11",
  "confidence": 0.95,
  "learned_from": "pattern_analysis"
}
```

## Migration Path

1. **Phase 1-2**: Silent learning, no UI impact
2. **Phase 3**: Start showing suggestions (opt-in)
3. **Phase 4**: Optional autonomous actions (user controls)
4. **Phase 5**: Progressive autonomy based on user comfort

## Success Metrics

- **Learning Accuracy**: Pattern recognition accuracy >85%
- **Suggestion Acceptance**: User accepts >70% of suggestions
- **Time Savings**: Reduce setup time by 40%+ through automation
- **Performance**: Learning operations add <5% overhead
- **User Satisfaction**: Positive feedback on autonomy level

## Files to Create/Modify

### New Files

- `src-tauri/src/domains/learning/` (entire domain)
- `src-tauri/src/migrations/mXXXXXX_create_learning_tables.rs`
- `src/lib/domains/learning/` (entire domain)
- `src-tauri/src/domains/autonomy/` (autonomy system)
- `src-tauri/src/domains/automation/workflow_engine.rs`

### Modified Files

- `src-tauri/src/lib.rs`: Register new commands
- `src-tauri/src/domains/automation/commands.rs`: Add embedded workflow commands
- `src-tauri/src/database.rs`: Add learning tables
- `src/lib/domains/projects/`: Integrate learning suggestions
- `src/lib/domains/sdk/`: Add learned preference selection
- `src/lib/domains/terminal/`: Add command pattern suggestions

This plan transforms Portal Desktop into a truly intelligent, self-learning developer toolkit while maintaining performance and user control.

### To-dos

- [ ] Create ML intensity configuration system (performance slider: Fast/Light/Medium/Deep learning modes)
- [ ] Implement code pattern analyzer for learning best practices and patterns across projects
- [ ] Design lightweight embedded automation engine for Tauri (alternative to n8n)