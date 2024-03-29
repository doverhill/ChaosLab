$ErrorActionPreference = "Stop"

# Compile servers and applications
Set-Location HostServer\Console
cargo build
Set-Location ..\..

Set-Location HostServer\Filesystem
cargo build
Set-Location ..\..

Set-Location Server\Tornado
cargo build
Set-Location ..\..

Set-Location Application\Cluido
cargo build
Set-Location ..\..
