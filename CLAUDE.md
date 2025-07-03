# Development Partnership

We're building production-quality code together. Your role is to create maintainable, efficient solutions while catching potential issues early.

When you seem stuck or overly complex, I'll redirect you - my guidance helps you stay on track.

## 🚨 AUTOMATED CHECKS ARE MANDATORY

**ALL hook issues are BLOCKING - EVERYTHING must be ✅ GREEN!**  
No errors. No formatting issues. No linting problems. Zero tolerance.  
These are not suggestions. Fix ALL issues before continuing.

## CRITICAL WORKFLOW - ALWAYS FOLLOW THIS!

### Research → Plan → Implement

**NEVER JUMP STRAIGHT TO CODING!** Always follow this sequence:

1. **Research**: Explore the codebase, understand existing patterns
2. **Plan**: Create a detailed implementation plan and verify it with me
3. **Implement**: Execute the plan with validation checkpoints

When asked to implement any feature, you'll first say: "Let me research the codebase and create a plan before implementing."

For complex architectural decisions or challenging problems, use **"ultrathink"** to engage maximum reasoning capacity. Say: "Let me ultrathink about this architecture before proposing a solution."

### USE MULTIPLE AGENTS!

_Leverage subagents aggressively_ for better results:

- Spawn agents to explore different parts of the codebase in parallel
- Use one agent to write tests while another implements features
- Delegate research tasks: "I'll have an agent investigate the database schema while I analyze the API structure"
- For complex refactors: One agent identifies changes, another implements them

Say: "I'll spawn agents to tackle different aspects of this problem" whenever a task has multiple independent parts.

### Reality Checkpoints

**Stop and validate** at these moments:

- After implementing a complete feature
- Before starting a new major component
- When something feels wrong
- Before declaring "done"
- **WHEN HOOKS FAIL WITH ERRORS** ❌

Run: `cargo fmt && cargo test && cargo clippy`

> Why: You can lose track of what's actually working. These checkpoints prevent cascading failures.

### 🚨 CRITICAL: Hook Failures Are BLOCKING

**When hooks report ANY issues (exit code 2), you MUST:**

1. **STOP IMMEDIATELY** - Do not continue with other tasks
2. **FIX ALL ISSUES** - Address every ❌ issue until everything is ✅ GREEN
3. **VERIFY THE FIX** - Re-run the failed command to confirm it's fixed
4. **CONTINUE ORIGINAL TASK** - Return to what you were doing before the interrupt
5. **NEVER IGNORE** - There are NO warnings, only requirements

This includes:

- Formatting issues (rustfmt)
- Linting violations (clippy)
- Forbidden patterns (unsafe blocks, unwrap(), panic!())
- ALL other checks

Your code must be 100% clean. No exceptions.

**Recovery Protocol:**

- When interrupted by a hook failure, maintain awareness of your original task
- After fixing all issues and verifying the fix, continue where you left off
- Use the todo list to track both the fix and your original task

## Working Memory Management

### When context gets long:

- Re-read this CLAUDE.md file
- Summarize progress in a PROGRESS.md file
- Document current state before major changes

### Maintain TODO.md:

```
## Current Task
- [ ] What we're doing RIGHT NOW

## Completed
- [x] What's actually done and tested

## Next Steps
- [ ] What comes next
```

## Rust-Specific Rules

### FORBIDDEN - NEVER DO THESE:

- **NO unwrap()** or **expect()** in production code - use proper error handling!
- **NO panic!()** - use `Result<T, E>` for error handling!
- **NO** keeping old and new code together
- **NO** migration functions or compatibility layers
- **NO** versioned function names (process_v2, handle_new)
- **NO** custom error types without clear hierarchy
- **NO** TODOs in final code
- **NO** unsafe blocks without explicit justification

> **AUTOMATED ENFORCEMENT**: The clippy hook will BLOCK commits that violate these rules.  
> When you see `❌ FORBIDDEN PATTERN`, you MUST fix it immediately!

### Required Standards:

- **Delete** old code when replacing it
- **Meaningful names**: `user_id` not `id`
- **Early returns** to reduce nesting
- **Concrete types** from constructors: `fn new() -> Self`
- **Proper error handling**: `Result<T, E>` and `?` operator
- **Table-driven tests** for complex logic
- **Channels for synchronization**: Use `tokio::sync` or `std::sync::mpsc`
- **Async/await** for I/O operations, not blocking threads

### Example Patterns:

```rust
// ✅ GOOD: Proper error handling
fn parse_config(path: &Path) -> Result<Config, ConfigError> {
    let content = fs::read_to_string(path)?;
    toml::from_str(&content).map_err(ConfigError::Parse)
}

// ❌ BAD: Using unwrap
fn parse_config(path: &Path) -> Config {
    let content = fs::read_to_string(path).unwrap();
    toml::from_str(&content).unwrap()
}

// ✅ GOOD: Constructor pattern
impl Server {
    fn new(config: Config) -> Self {
        Self { config }
    }
}

// ✅ GOOD: Async with proper error handling
async fn fetch_data(url: &str) -> Result<Data, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let data = response.json().await?;
    Ok(data)
}

// ✅ GOOD: Channel synchronization
use tokio::sync::mpsc;

async fn worker(mut rx: mpsc::Receiver<Task>) {
    while let Some(task) = rx.recv().await {
        task.execute().await;
    }
}
```

## Implementation Standards

### Our code is complete when:

- ✅ All linters pass with zero issues
- ✅ All tests pass
- ✅ Feature works end-to-end
- ✅ Old code is deleted
- ✅ Documentation on all public items

### Testing Strategy

- Complex business logic → Write tests first
- Red -> Green -> Refactor
- Simple CRUD → Write tests after
- Hot paths → Add benchmarks
- Skip tests for main() and simple CLI parsing

### Project Structure

```
src/
├── main.rs       # Application entrypoint
├── lib.rs        # Library root
├── bin/          # Additional binaries
├── modules/      # Core modules
├── error.rs      # Error types
└── config.rs     # Configuration
tests/            # Integration tests
benches/          # Benchmarks
```

## Problem-Solving Together

When you're stuck or confused:

1. **Stop** - Don't spiral into complex solutions
2. **Delegate** - Consider spawning agents for parallel investigation
3. **Ultrathink** - For complex problems, say "I need to ultrathink through this challenge" to engage deeper reasoning
4. **Step back** - Re-read the requirements
5. **Simplify** - The simple solution is usually correct
6. **Ask** - "I see two approaches: [A] vs [B]. Which do you prefer?"

My insights on better approaches are valued - please ask for them!

## Performance & Security

### **Measure First**:

- No premature optimization
- Benchmark before claiming something is faster
- Use `cargo bench` for real bottlenecks
- Profile with `perf` or `cargo flamegraph`

### **Security Always**:

- Validate all inputs
- Use `ring` or `rustls` for cryptography
- Prepared statements for SQL (never concatenate!)
- Sanitize user input

## Communication Protocol

### Progress Updates:

```
✓ Implemented authentication (all tests passing)
✓ Added rate limiting
✗ Found issue with token expiration - investigating
```

### Suggesting Improvements:

"The current approach works, but I notice [observation].
Would you like me to [specific improvement]?"

## Working Together

- This is always a feature branch - no backwards compatibility needed
- When in doubt, we choose clarity over cleverness
- **REMINDER**: If this file hasn't been referenced in 30+ minutes, RE-READ IT!

Avoid complex abstractions or "clever" code. The simple, obvious solution is probably better, and my guidance helps you stay focused on what matters.
