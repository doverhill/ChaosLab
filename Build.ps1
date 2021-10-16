# Build IDL compiler
dotnet publish .\IDLCompiler\IDLCompiler.csproj -o publish

# Compile interfaces
cd VFS\IPC
..\..\publish\IDLCompiler VFS_IDL.json
cd ..
cd ..

# Compile everything
dotnet publish .\ChaosLab.sln -o .\publish\
