# Build IDL compiler
dotnet publish -c Release .\IDLCompiler\IDLCompiler.csproj -o .\build

# Compile protocols

Set-Location Protocol\Console
..\..\build\IDLCompiler Console.IDL.json
Set-Location ..\..

Set-Location Protocol\Data
..\..\build\IDLCompiler Data.IDL.json
Set-Location ..\..

Set-Location Protocol\Storage
..\..\build\IDLCompiler Storage.IDL.json
Set-Location ..\..

Set-Location Protocol\Filesystem
..\..\build\IDLCompiler Filesystem.IDL.json
Set-Location ..\..

Set-Location Protocol\Tornado
..\..\build\IDLCompiler Tornado.IDL.json
Set-Location ..\..
