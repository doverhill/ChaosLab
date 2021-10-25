# Build IDL compiler
dotnet publish .\IDLCompiler\IDLCompiler.csproj -o publish

# Compile interfaces
# VFS Library
#cd VFS\IPC
#..\..\publish\IDLCompiler client ..\VFS_IDL.json
#cd ..\..

# VFS Server
#cd VFSServer\IPC
#..\..\publish\IDLCompiler server ..\..\VFS\VFS_IDL.json
#cd ..\..

# Compile everything
dotnet publish .\ChaosLab.sln -o .\publish\
