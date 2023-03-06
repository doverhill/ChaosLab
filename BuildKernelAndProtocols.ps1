$ErrorActionPreference = "Stop"

# Build Storm
dotnet publish -c Release .\Storm\StormHost\StormHost.csproj -o .\build

# Build protocols
.\BuildProtocols.ps1

