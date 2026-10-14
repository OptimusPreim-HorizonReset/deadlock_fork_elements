Preparing a PR locally (due to sandbox restrictions this must be run on the developer machine).

1) Run the test suite locally before committing
   - Set deterministic test threads: $env:RUST_TEST_THREADS = '1'
   - Run: .\run_checks.ps1
   - Verify: all tests pass and headless smoke runs are acceptable.

2) Use the helper script to create a branch, commit, and push
   - From repository root (F:\Games\deadlock_fork_elements):
     PowerShell: .\commit_and_push.ps1
   - This creates or checks out branch "feature/flow-m1-roles-clippy-tests", commits staged changes with the prepared commit message (includes Co-authored-by trailer), and attempts to push to origin.

3) If push fails (credentials/remote not configured):
   - The branch exists locally. Run:
       git remote -v   # inspect remotes
       git push -u origin feature/flow-m1-roles-clippy-tests
   - Or create a patch: git format-patch -1 HEAD

4) PR description (suggested):
   Title: M1: Expand role updates and map roles -> behaviour; add tests
   Body: Summary of changes (copy the commit message from commit_and_push.ps1). Include notes:
     - This patch expands M1 role mapping and applies low-amplitude role effects. These are intentionally conservative; tuning is expected during M1->M3.
     - The changes are feature-flag gated by `enable_flow_dynamics_model` and default behaviour is unchanged unless the flag is enabled.
     - Local test coverage added for role normalization and basic role-effect verification.
     - Known issue: Windows release builds may intermittently fail due to file-locking on target/release/barnes-hut.exe; run_checks.ps1 builds to a temp target as a mitigation.

5) After PR is created: request CI (if available) to run full test/clippy/build matrix. If any Clippy failures appear, address them in follow-up commits (scope: small, surgical fixes).