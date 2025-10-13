# GitHub Actions Workflows

This directory contains GitHub Actions workflows for CI/CD automation.

## Workflows

### CI Workflow (`ci.yml`)

**Triggers:**
- Pull requests to `master` or `main` branches
- Pushes to `master` or `main` branches
- Manual trigger via GitHub Actions UI

**Jobs:**
- **Test Suite**: Runs all tests on Linux, macOS, and Windows
- **Rustfmt**: Checks code formatting
- **Clippy**: Runs linting checks

**Purpose:** Ensures code quality and tests pass before merging PRs.

### Release Workflow (`release.yml`)

**Triggers:**
- Push of tags matching `v*.*.*` pattern (e.g., `v1.0.0`, `v2.1.3`)
- Manual trigger via GitHub Actions UI

**Jobs:**
- **Build**: Compiles binaries for multiple platforms
  - Linux x86_64
  - Linux ARM64
  - macOS x86_64 (Intel)
  - macOS ARM64 (Apple Silicon)
  - Windows x86_64

- **Release**: Creates GitHub release with all binaries (only on tags)
- **Summary**: Displays build summary

**Artifacts:**
- `minipx-cli-{target}.tar.gz` or `.zip` - CLI binaries
- `minipx-web-{target}.tar.gz` or `.zip` - Web binaries

## Usage

### Running CI on Pull Requests

CI automatically runs when you create or update a pull request:

```bash
git checkout -b feature/my-feature
# Make changes
git commit -am "Add new feature"
git push origin feature/my-feature
# Create PR on GitHub - CI runs automatically
```

### Creating a Release

1. **Create and push a tag:**
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```

2. **Wait for the workflow to complete** (check Actions tab on GitHub)

3. **Release is created automatically** with all binaries attached

### Manual Build Trigger

You can manually trigger builds without creating a release:

1. Go to GitHub Actions tab
2. Select "Release" workflow
3. Click "Run workflow"
4. Select branch and optionally specify version
5. Click "Run workflow" button

Artifacts will be available for download from the workflow run page.

## Build Targets

| Platform | Target | Binary Format |
|----------|--------|---------------|
| Linux (x64) | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Linux (ARM64) | `aarch64-unknown-linux-gnu` | `.tar.gz` |
| macOS (Intel) | `x86_64-apple-darwin` | `.tar.gz` |
| macOS (Apple Silicon) | `aarch64-apple-darwin` | `.tar.gz` |
| Windows (x64) | `x86_64-pc-windows-msvc` | `.zip` |

## Caching

Both workflows use cargo caching to speed up builds:
- Cargo registry
- Cargo index
- Build artifacts

Cache keys are based on OS, target, and `Cargo.lock` hash.

## Requirements

### For Contributors
- Code must pass all tests
- Code must be formatted with `cargo fmt`
- Code must pass `cargo clippy` without warnings

### For Releases
- Tag must follow semantic versioning: `v{major}.{minor}.{patch}`
- Example: `v1.0.0`, `v2.1.3`, `v0.1.0`

## Troubleshooting

### CI Fails on Formatting
Run locally before pushing:
```bash
cargo fmt --all
```

### CI Fails on Clippy
Run locally to see warnings:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Release Build Fails
Check the Actions tab for detailed logs. Common issues:
- Missing dependencies for cross-compilation
- Test failures
- Build errors on specific platforms

### Manual Workflow Not Appearing
Ensure you're looking at the correct branch. Manual workflows only appear when viewing the branch that contains them.

## Development

To test workflow changes locally, you can use:
- [act](https://github.com/nektos/act) - Run GitHub Actions locally
- [actionlint](https://github.com/rhysd/actionlint) - Lint workflow files

Example:
```bash
# Install actionlint
go install github.com/rhysd/actionlint/cmd/actionlint@latest

# Lint workflows
actionlint .github/workflows/*.yml
```
