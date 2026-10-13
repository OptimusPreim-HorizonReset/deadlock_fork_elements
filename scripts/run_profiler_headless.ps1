param(
    [int]$MaxFrames = 200,
    [int]$FramesPerCycle = 50,
    [switch]$StepComponents
)

$env:PROFILE_RUNTIME = '1'
$env:PROFILE_MAX_FRAMES = "$MaxFrames"
$env:PROFILE_FRAMES = "$FramesPerCycle"
if ($StepComponents) { $env:PROFILE_STEP_COMPONENTS = '1' } else { $env:PROFILE_STEP_COMPONENTS = '0' }

Write-Host "Starting headless profiler: PROFILE_MAX_FRAMES=$MaxFrames, PROFILE_FRAMES=$FramesPerCycle, PROFILE_STEP_COMPONENTS=$($env:PROFILE_STEP_COMPONENTS)"

# Run release build for more realistic profiling
cargo run --release
