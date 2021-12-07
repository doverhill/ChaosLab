# Build IDL compiler
dotnet publish .\IDLCompiler\IDLCompiler.csproj -o publish

# Compile protocols

# FileSystem
Set-Location rust\Protocol\FileSystem\src
..\..\..\..\publish\IDLCompiler ..\FileSystem.IDL.json
Set-Location ..\..\..\..

# FrameBufferDisplay
Set-Location rust\Protocol\FrameBufferDisplay\src
..\..\..\..\publish\IDLCompiler ..\FrameBufferDisplay.IDL.json
Set-Location ..\..\..\..

# Tornado
Set-Location rust\Protocol\Tornado\src
..\..\..\..\publish\IDLCompiler ..\Tornado.IDL.json
Set-Location ..\..\..\..

