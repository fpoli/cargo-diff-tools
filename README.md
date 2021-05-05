# Cargo diff-tools

Run `cargo check` and `cargo clippy` hiding the warning messages whose primary line is not included in a `git diff`. Useful in large projects to hide warning messages that are probably not related to the changes made by a pull request.

Inspired by [`Patryk27/clippy-dirty`](https://github.com/Patryk27/clippy-dirty).

## Install

```bash
cargo install cargo-diff-tools
```

## Examples

Run `cargo clippy` hiding the warning messages whose primary line is not included in a `git origin/master HEAD`:

```bash
cargo-clippy-diff origin/master HEAD
```

The same, for `cargo check`:

```bash
cargo-check-diff origin/master HEAD
```

Various `git diff` arguments are supported:

```bash
cargo-clippy-diff HEAD      # internally calls `git diff HEAD`
cargo-clippy-diff --staged  # internally calls `git diff --staged`
cargo-clippy-diff first-branch...second-branch origin/master  # and so on
```

Place `cargo check` arguments after a `--`:

```bash
cargo-check-diff HEAD -- --all-features
```

Place `cargo clippy` arguments after a `--` (note that the second `--` is one of `clippy`'s arguments):

```bash
cargo-clippy-diff HEAD -- --all-features -- -D clippy::lint_name
```

To display diagnostics as JSON objects, use `--output=json`:

```bash
cargo-clippy-diff --output=json origin/master HEAD
```

To display diagnostics as [workflow commands in GitHub Actions](https://docs.github.com/en/actions/reference/workflow-commands-for-github-actions#setting-a-warning-message) (useful to automatically add comments to pull requests), use `--output=github`:

```bash
git fetch origin "$GITHUB_BASE_REF"
HEAD_REPO="${{ github.event.pull_request.head.repo.full_name }}"
git fetch "$HEAD_REPO" "$GITHUB_HEAD_REF"
FULL_HEAD_REF="$HEAD_REPO/$GITHUB_HEAD_REF"
cargo-clippy-diff --output=github $(git merge-base "origin/$GITHUB_BASE_REF" "$FULL_HEAD_REF") "$FULL_HEAD_REF"
# Example output "::warning file=lib.rs,line=4,col=2::Missing semicolon"
```

For other `cargo` commands, `filter-by-diff` can be used to filter any stream of JSON diagnostics:

```bash
cargo build --message-format=json-diagnostic-rendered-ansi \
    | filter-by-diff --output=rendered origin/master HEAD
```
