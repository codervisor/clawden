---
status: planned
created: 2026-03-11
priority: critical
tags:
- core
- adapter
- worker
- ai-infra
parent: 060-ai-infra-pivot
created_at: 2026-03-11T06:12:38.288351Z
updated_at: 2026-03-11T06:12:38.288351Z
---

# Worker Adapter Interface — Coding Tool CRI

## Overview

Define the `WorkerAdapter` trait — the CRI (Container Runtime Interface) pattern applied to AI coding tools. Each coding tool (Claude Code, Codex CLI, Copilot CLI, Gemini CLI, OpenCode) gets a thin adapter that knows how to launch it, send it tasks, capture output, and check health.

### Why a Trait

Same reason the existing `ClawAdapter` trait works well: each tool has different CLI flags, output formats, and execution models. The trait abstracts these differences behind a uniform interface so ClawDen's core doesn't need tool-specific logic.

## Design

### WorkerAdapter Trait

```rust
#[async_trait]
pub trait WorkerAdapter: Send + Sync {
    fn metadata(&self) -> WorkerMetadata;

    /// Launch the worker process, return a handle
    async fn start(&self, config: &WorkerConfig) -> Result<ProcessHandle>;

    /// Gracefully stop the worker
    async fn stop(&self, handle: &ProcessHandle) -> Result<()>;

    /// Check if the worker process is alive and responsive
    async fn health(&self, handle: &ProcessHandle) -> Result<HealthStatus>;

    /// Send a task and wait for result
    async fn dispatch(&self, handle: &ProcessHandle, task: &Task) -> Result<TaskResult>;

    /// Stream real-time output from the worker
    async fn stream_output(&self, handle: &ProcessHandle) -> Result<LogStream>;
}
```

### WorkerMetadata

```rust
pub struct WorkerMetadata {
    pub name: &'static str,           // "claude-code"
    pub display_name: &'static str,   // "Claude Code"
    pub binary: &'static str,         // "claude"
    pub install_hint: &'static str,   // "npm install -g @anthropic-ai/claude-code"
    pub supports_json_output: bool,
    pub supports_streaming: bool,
    pub supports_session_resume: bool,
}
```

### Task / TaskResult

```rust
pub struct Task {
    pub id: String,
    pub prompt: String,
    pub project_dir: PathBuf,
    pub timeout: Option<Duration>,
    pub allowed_tools: Option<Vec<String>>,
}

pub struct TaskResult {
    pub task_id: String,
    pub status: TaskStatus,        // Completed, Failed, TimedOut
    pub output: String,            // captured stdout/result
    pub duration: Duration,
    pub cost: Option<TaskCost>,    // tokens/dollars if available
}
```

### Claude Code Adapter (first implementation)

```rust
pub struct ClaudeCodeAdapter;

impl WorkerAdapter for ClaudeCodeAdapter {
    // dispatch: runs `claude -p "task prompt" --output-format json`
    // in the project directory, captures JSON output
    // health: checks `claude --version` / process alive
    // stream_output: pipes stdout in real-time
}
```

Execution model:
- `claude -p "prompt" --output-format json` for one-shot tasks
- Parse JSON output for structured result
- Respect `--allowedTools` if `task.allowed_tools` is set
- Working directory set to `task.project_dir`

### Worker Descriptor (mirrors RuntimeDescriptor pattern)

Static metadata array like `DESCRIPTORS` in `runtime_descriptor.rs`:

```rust
pub static WORKER_DESCRIPTORS: &[WorkerDescriptor] = &[
    WorkerDescriptor {
        id: CodingTool::ClaudeCode,
        name: "claude-code",
        display_name: "Claude Code",
        binary: "claude",
        install_check: "claude --version",
        install_hint: "npm i -g @anthropic-ai/claude-code",
        supports_json_output: true,
    },
    WorkerDescriptor {
        id: CodingTool::CodexCli,
        name: "codex",
        display_name: "Codex CLI",
        binary: "codex",
        install_check: "codex --version",
        install_hint: "npm i -g @openai/codex",
        supports_json_output: true,
    },
    // ... more tools
];
```

## Plan

- [ ] Define `WorkerAdapter` trait in `clawden-core`
- [ ] Define `WorkerDescriptor`, `Task`, `TaskResult` types
- [ ] Create `clawden-workers` crate (or module in `clawden-adapters`)
- [ ] Implement `ClaudeCodeAdapter` — launch, dispatch, capture
- [ ] Add `CodingTool` enum with descriptor registry
- [ ] Implement `CodexCliAdapter` as second adapter
- [ ] Integration test: dispatch task to Claude Code, verify output capture

## Test

- [ ] `ClaudeCodeAdapter::dispatch()` launches claude process and captures JSON output
- [ ] `ClaudeCodeAdapter::health()` correctly reports alive/dead status
- [ ] Worker descriptor registry returns correct metadata for each tool
- [ ] Unknown/uninstalled tool returns actionable error with install hint
