# Build IDL compiler
dotnet publish .\IDLCompiler\IDLCompiler.csproj -o publish

# Compile protocols

# fs
Set-Location rust\protocol\fs\src
..\..\..\..\publish\IDLCompiler ..\fs_idl.json
Set-Location ..\..\..\..

# fs
Set-Location rust\protocol\vfs\src
..\..\..\..\publish\IDLCompiler ..\vfs_idl.json
Set-Location ..\..\..\..

# Compile everything
dotnet publish .\ChaosLab.sln -o .\publish\

# Run Storm
.\publish\DebugHost.exe