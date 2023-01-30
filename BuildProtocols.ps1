# Build IDL compiler
dotnet publish -c Release .\IDLCompiler\IDLCompiler.csproj -o .\build

# Compile protocols

# FileSystem
Set-Location Protocol\Console
..\..\build\IDLCompiler Console.IDL.json
Set-Location ..\..

# FileSystem
Set-Location Protocol\Storage
..\..\build\IDLCompiler Storage.IDL.json
Set-Location ..\..

# FileSystem
Set-Location Protocol\Tornado
..\..\build\IDLCompiler Tornado.IDL.json
Set-Location ..\..