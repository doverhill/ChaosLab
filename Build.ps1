$ErrorActionPreference = "Stop"

# Build Storm
dotnet publish -c Release .\Storm\StormHost\StormHost.csproj -o .\build

.\BuildProtocols.ps1

# Compile servers and applications
Set-Location HostServer\Console
cargo build
Set-Location ..\..

Set-Location Server\Tornado
cargo build
Set-Location ..\..

