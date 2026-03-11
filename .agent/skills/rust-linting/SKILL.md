---
name: rust-linting
description: Always run cargo fmt and cargo clippy together to ensure code quality and consistent style. When user asks to check or lint, use this combined command.
---

# Rust Linting and Formatting

To ensure the codebase maintains high standards and consistent style, you MUST always run both `cargo fmt` and `cargo clippy` when making changes to Rust code.

## Workflow

When you are ready to verify your changes, run the following combined command:

```bash
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings
```

### Why both?

1.  **`cargo fmt`**: Ensures the code adheres to the project's formatting rules. `cargo clippy` does not check for formatting issues like trailing whitespaces or indentation.
2.  **`cargo clippy`**: Catches common mistakes, anti-patterns, and potential bugs.

## Common Clippy Fixes

- **Collapsible `if`**: Instead of nested `if` statements, use `.filter()` on an `Option` or combine conditions if possible.
- **`is_none_or`**: Prefer `opt.is_none_or(|x| condition(x))` over `opt.map_or(true, |x| condition(x))`.
- **`is_some_and`**: Prefer `opt.is_some_and(|x| condition(x))` over `opt.map_or(false, |x| condition(x))`.

## Verification

Before declaring a task as complete, always ensure that `cargo fmt --all --check` and `cargo clippy --all-targets --all-features -- -D warnings` both pass with exit code 0.
