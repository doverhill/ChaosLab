$ErrorActionPreference = "Stop"

$env:RUST_BACKTRACE=1
$env:RUST_BACKTRACE="full"

# First, build everything
.\BuildApps.ps1

# Start Storm
.\build\StormHost.exe
