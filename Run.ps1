$ErrorActionPreference = "Stop"

# First, build everything
.\Build.ps1

# Start Storm
.\build\StormHost.exe
