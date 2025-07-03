---
allowed-tools: all
description: Context handoff prompts and interactive help system
---

# Context & Help System

Context handoff for LLM transitions and interactive help for commands/workflows.

**Usage:**
- `/prompt [focus_area]` - Generate context handoff prompt
- `/prompt help [command|topic]` - Get help on commands/workflows

**Examples:**
- `/prompt debugging` - Generate debugging context handoff
- `/prompt help tdd` - Get detailed help on TDD workflow
- `/prompt help workflow` - Show development workflow guidance

## Context Handoff (`/prompt [focus]`)

**Generates efficient handoff prompts including:**
- Current working directory and git status
- Active tasks and recent changes
- Next logical steps and current state
- Key constraints from CLAUDE.md

**Optimized for low context:**
- No unnecessary file reads
- Focuses on forward momentum
- Essential state information only
- Ready for fresh LLM session

## Help System (`/prompt help [topic]`)

### **Available Help Topics:**

**Commands:**
- `tdd` - Test-driven development workflow and actions
- `check` - Quality verification and validation
- `next` - Feature implementation process
- `claude-md` - Instruction file maintenance

**Workflows:**
- `workflow` - Complete development process
- `learning` - Knowledge preservation and skill building
- `quality` - Code quality and validation standards

**Quick Reference:**
- `commands` - List all available commands
- `examples` - Common usage patterns and examples

### **Command Quick Reference:**
- **`/check`** - Comprehensive quality verification
- **`/next <task>`** - Structured feature implementation  
- **`/tdd <feature|action>`** - TDD workflow and test management
- **`/claude-md [action]`** - Maintain instruction file
- **`/prompt [focus|help]`** - Context handoff or help

Execute `/prompt help workflow` for complete development guidance.