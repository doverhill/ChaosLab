using IDLCompiler;
using System.Text.Json;

if (args.Length < 1)
{
    Console.WriteLine("Error: Use with <FileName.IDL.json>");
    return;
}

var filename = args[0];
var fileContents = File.ReadAllText(filename);
JsonSerializerOptions options = new()
{
    AllowTrailingCommas = true,
    IncludeFields = true,
    WriteIndented = false,
    PropertyNamingPolicy = null,
    DictionaryKeyPolicy = null,
    ReadCommentHandling = JsonCommentHandling.Skip,
    NumberHandling = System.Text.Json.Serialization.JsonNumberHandling.Strict,
    Converters = { new System.Text.Json.Serialization.JsonStringEnumConverter() }
};

IDL idl = null;
try
{
    idl = JsonSerializer.Deserialize<IDL>(fileContents, options);
}
catch (Exception e)
{
    Console.WriteLine("Error: Failed to read IDL file: " + e.Message);
}

if (idl == null)
{
    Console.WriteLine("Error: Failed to read IDL file. File empty?");
    return;
}

if (idl.Interface == null)
{
    Console.WriteLine("Error: IDL file is missing Interface description");
    return;
}

Console.WriteLine("Building interface " + idl.Interface.Name + " (version " + idl.Interface.Version + ")");

// set up lib
var libStream = new StreamWriter(File.Create("lib.rs"));
libStream.WriteLine("#[macro_use]");
libStream.WriteLine("extern crate lazy_static;");
libStream.WriteLine();

// emit types
TypeEmitter.Reset();
if (idl.Types != null)
{
    Console.WriteLine(idl.Types.Count + " type(s)");

    libStream.WriteLine("mod types;");
    libStream.WriteLine("pub use types::*;");
    libStream.WriteLine();

    foreach (var type in idl.Types)
    {
        Console.WriteLine("  Emitting type " + type.Name);
        TypeEmitter.Emit(idl, type);
    }
}

// emit client to server calls
CallEmitter.Reset();
var emittedClientToServerCall = false;
if (idl.ClientToServerCalls != null)
{
    Console.WriteLine(idl.ClientToServerCalls.Count + " client->server call(s)");

    foreach (var call in idl.ClientToServerCalls)
    {
        Console.WriteLine("  Emitting client->server call " + call.Name);
        CallEmitter.Emit(CallEmitter.Direction.ClientToServer, idl, call);
        emittedClientToServerCall = true;
    }

    if (emittedClientToServerCall)
    {
        libStream.WriteLine("mod client_to_server_calls;");
        libStream.WriteLine();
    }
}

// emit server to client calls
var emittedServerToClientCall = false;
if (idl.ServerToClientCalls != null)
{
    Console.WriteLine(idl.ServerToClientCalls.Count + " server->client call(s)");

    foreach (var call in idl.ServerToClientCalls)
    {
        Console.WriteLine("  Emitting server->client call " + call.Name);
        CallEmitter.Emit(CallEmitter.Direction.ServerToClient, idl, call);
        emittedServerToClientCall = true;
    }

    if (emittedServerToClientCall)
    {
        libStream.WriteLine("mod server_to_client_calls;");
        libStream.WriteLine();
    }
}

// emit "server"
ClientServerEmitter.Emit(ClientServerEmitter.Side.Server, idl, idl.ClientToServerCalls, idl.ServerToClientCalls);
libStream.WriteLine("mod server;");
libStream.WriteLine("pub use server::" + idl.Interface.Name + "Server;");
libStream.WriteLine("pub use server::" + idl.Interface.Name + "ServerImplementation;");
libStream.WriteLine();

// emit "client"
ClientServerEmitter.Emit(ClientServerEmitter.Side.Client, idl, idl.ServerToClientCalls, idl.ClientToServerCalls);
libStream.WriteLine("mod client;");
libStream.WriteLine("pub use client::" + idl.Interface.Name + "Client;");
libStream.WriteLine("pub use client::" + idl.Interface.Name + "ClientImplementation;");
libStream.WriteLine();

if (!emittedClientToServerCall && !emittedServerToClientCall)
{
    Console.WriteLine("Warning: No calls emitted!");
}

libStream.Close();

Console.WriteLine("Done");
