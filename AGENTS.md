# Agent Instructions

## Context
You are working on "OmniTagger", a desktop application for extracting AI prompts from screen captures.
The project source code and documentation are located in the `omni-tagger/` directory.

## Documentation (Mandatory)
**CRITICAL**: Before starting any task, you must:
1. Read `omni-tagger/docs/SPECIFICATION.md` to understand the functional and technical requirements.
2. Read `omni-tagger/docs/TODO.md` to understand the current project status and identify the next steps.

## Tech Stack
- **Framework**: Tauri v2
- **Backend**: Rust
- **Frontend**: React + TypeScript
- **Styling**: Tailwind CSS v4

## Code Conventions
- Follow standard Rust conventions (run `cargo clippy` and `cargo fmt`).
- Follow standard TypeScript/React conventions.
- **Path Awareness**: Be aware that the Tauri project root is `omni-tagger/src-tauri` for Rust commands, and `omni-tagger/` for Node commands.

## Workflow Best Practices
1. **Plan First**: Always analyze the `docs/` and existing code before writing a plan.
2. **Verify Changes**: Use `read_file` or test commands to verify your changes after editing files.
3. **Update Documentation**:
   - After completing a task, you MUST update `omni-tagger/docs/TODO.md` to reflect the progress.
   - If you encounter implementation details that differ from `SPECIFICATION.md`, update the spec or ask for clarification.
