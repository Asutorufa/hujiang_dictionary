---
name: linting
description: Always run linting and formatting for both Rust and Frontend to ensure code quality and consistent style across the stack.
---

# Linting and Formatting

To ensure the codebase maintains high standards and consistent style, you MUST always run linting and formatting tools when making changes.

## Backend (Rust)

Run the following combined command to verify Rust changes:

```bash
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings
```

### Why both?

1.  **`cargo fmt`**: Ensures the code adheres to the project's formatting rules.
2.  **`cargo clippy`**: Catches common mistakes, anti-patterns, and potential bugs.

## Frontend (Web)

Run the following combined command to verify Frontend changes in the `web` directory:

```bash
npm run lint && npx tsc --noEmit && npx prettier --check .
```

To fix formatting and linting issues automatically:

```bash
npm run lint:fix && npx prettier --write .
```

### Tools used:

1.  **ESLint**: Checks for code quality and patterns.
2.  **TypeScript (`tsc`)**: Verifies type safety.
3.  **Prettier**: Ensures consistent style and formatting.

## Common Clippy Fixes

- **Collapsible `if`**: Instead of nested `if` statements, use `.filter()` on an `Option` or combine conditions if possible.
- **`is_none_or`**: Prefer `opt.is_none_or(|x| condition(x))` over `opt.map_or(true, |x| condition(x))`.
- **`is_some_and`**: Prefer `opt.is_some_and(|x| condition(x))` over `opt.map_or(false, |x| condition(x))`.

## Scenarios

- **Before Committing**: Always run these checks to avoid CI failures.
- **After Refactoring**: Run to ensure no new anti-patterns or type errors were introduced.
- **When Adding Dependencies**: Verify that everything still compiles and adheres to standards.

## Examples

- **Good**: `cargo clippy` passes with 0 warnings/errors and `cargo fmt` reports no changes.
- **Bad**: Committing with `#[allow(clippy::all)]` or ignoring formatting warnings.

## Verification

Before declaring a task as complete, always ensure that all linting and type checks pass with exit code 0 for both backend and frontend if applicable.
