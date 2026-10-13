# create_github_workflow.ps1
# Usage: run this locally in the repo root to create a GitHub Actions workflow
# that runs tests, clippy, build and a short headless smoke run and uploads logs
# as workflow artifacts.

param(
    [string]$RepoRoot = "."
)

Set-StrictMode -Version Latest
Push-Location $RepoRoot

$workflowDir = Join-Path $RepoRoot ".github\workflows"
if (-not (Test-Path $workflowDir)) {
    New-Item -ItemType Directory -Force -Path $workflowDir | Out-Null
}

$workflowPath = Join-Path $workflowDir "logs-ci.yml"
$workflow = @'
name: CI - Logs

on:
  push:
  pull_request:
  workflow_dispatch:

jobs:
  build_and_capture_logs:
    runs-on: ubuntu-latest
    env:
      RUST_TEST_THREADS: 1
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Run tests
        run: |
          cargo test --all --workspace 2>&1 | tee test-output.txt

      - name: Run clippy
        continue-on-error: true
        run: |
          cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tee clippy-output.txt

      - name: Format
        run: cargo fmt --all

      - name: Build release
        run: |
          cargo build --release 2>&1 | tee build-output.txt

      - name: Headless smoke run
        continue-on-error: true
        run: |
          cargo run -- --headless-test 2>&1 | tee headless-output.txt

      - name: Upload logs
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: logs
          path: |
            test-output.txt
            clippy-output.txt
            build-output.txt
            headless-output.txt
'@

Set-Content -Path $workflowPath -Value $workflow -Encoding UTF8
Write-Host "Created GitHub Actions workflow at $workflowPath"

Pop-Location
