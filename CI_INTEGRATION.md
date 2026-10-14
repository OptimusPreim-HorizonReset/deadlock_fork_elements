CI integration for automated log collection

This small guide explains how to install and use the GitHub Actions workflow that will produce the logs (test, clippy, build and headless) and upload them as artifacts for download.

1) Install the workflow (on your local machine):
   - Open PowerShell at the repository root (F:\Games\deadlock_fork_elements)
   - Run: .\create_github_workflow.ps1
   - This will create `.github\workflows\logs-ci.yml`.

2) Push the branch that contains the workflow (or merge to a branch that triggers Actions).
   - The workflow runs on push, pull_request, and can be manually triggered via "Run workflow" in the Actions tab (workflow_dispatch).

3) After workflow completes:
   - Open the repository Actions tab -> select the latest "CI - Logs" run -> Artifacts -> download "logs".
   - The artifact is a zip containing the following files at the root of the artifact:
     - test-output.txt
     - clippy-output.txt
     - build-output.txt
     - headless-output.txt

4) Post the downloaded files (or the parsed summaries) here and I will:
   - parse and summarize the failures (compile errors, failing tests, clippy warnings),
   - prepare minimal surgical patches for each issue,
   - iterate until cargo test, cargo clippy (-D warnings) and cargo build --release succeed.

Notes
- The workflow sets RUST_TEST_THREADS=1 to avoid race conditions for tests that touch the filesystem.
- The clippy and headless steps use `continue-on-error: true` so the workflow will still upload logs even if clippy or the headless smoke run fail. The tests/build steps are still run and their output is captured.
- If you prefer a different Rust toolchain or additional steps (cargo fmt check, coverage, etc.) tell me and I will update the workflow script.
