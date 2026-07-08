---
name: code-it-later
description: Scan codebase comments to find development crumbs, TODOs, and backlog items using the `codeitlater` CLI tool, and ask the user whether to clean or restore them after implementation.
---

# code-it-later Skill

Use this skill when you need to scan the codebase for developer flags, TODOs, bugs, or refactoring points (referred to as **crumbs**), or when you need to clean up/restore those flags after completing tasks.

## Crumb Syntax
Crumbs are comments in source code marked with the `:=` symbol (e.g., `//:= TODO: fix logic`).
- **Multi-line**: A crumb line ending with `...` appends the next line to the same crumb (e.g., `//:= step1... \n //:= step2`).
- **Ignored/Backlog**: Crumbs starting with `!` (e.g., `//:= !TODO: ...`) are ignored by default.
- **Keywords**: Custom prefixes followed by a colon (e.g., `TODO:`, `MARK:`, `BUG:`, `JIRA-123:`).

## How to Run the Tool
The `codeitlater` binary is installed locally and can be executed via terminal command execution.

### Common CLI Recipes

1. **Scan all active crumbs in the repository**:
   ```bash
   codeitlater
   ```

2. **Retrieve crumbs with code context (Optional)**:
   Use this only when you need surrounding code context (e.g. `N` lines above and below) to understand the crumb:
   ```bash
   codeitlater -r 3
   ```

3. **Parse crumbs programmatically as JSON (Highly recommended for programmatic parsing)**:
   Use this format to get structured output that can be easily parsed by scripts or agents:
   ```bash
   codeitlater -O json
   ```

4. **Filter by a specific keyword or query ignored items**:
   ```bash
   # Filter by specific keyword
   codeitlater -k TODO
   
   # Include ignored backlog items
   codeitlater --show-ignored
   ```

5. **Clean up crumbs**:
   After resolving a crumb's task, you can clean it up (delete) or restore it to a normal comment:
   ```bash
   # Delete the crumbs (using -y to bypass interactive prompt)
   codeitlater -D -y
   
   # Restore crumbs to normal comments (using -y to bypass interactive prompt)
   codeitlater -R -y
   ```

## Agent Guidelines
1. **Context Gathering**: When assigned a refactoring or implementation task, run `codeitlater` (without `-r` by default) to check for any relevant crumbs. If you need surrounding code context to understand the crumb or reason about the required changes, use `codeitlater -r <N>` (e.g. `codeitlater -r 3`).
2. **Check Configuration**: Always check if a `.codeitlater` file exists in the repository root directory. It may contain repository-specific arguments (like excluded paths or default keywords) that will be pre-loaded.
3. **Structured Parsing**: Prefer using `-O json` when you need to programmatically parse or inspect the exact lines and structures of crumbs in the codebase.
4. **Implementation**: Address the code comments flagged by the crumbs.
5. **Clean-up**: Once the code changes are complete and verified, **always ask the user whether they want to clean up (delete) or restore the crumbs**, and obtain their explicit preference before running `codeitlater -D -y` or `codeitlater -R -y`. Do not perform clean-up or restore automatically.
