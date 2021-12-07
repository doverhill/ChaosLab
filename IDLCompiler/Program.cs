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

//if (idl.Interface.InheritsFrom != null)
//{
//    Console.WriteLine("Inherits from " + idl.Interface.InheritsFrom);
//    var path = Path.Combine(Path.GetDirectoryName(filename), idl.Interface.InheritsFrom);
//    var baseContents = File.ReadAllText(path);
//    var baseIdl = JsonSerializer.Deserialize<IDL>(baseContents, options);

//    if (baseIdl.Types != null)
//    {
//        if (idl.Types == null)
//        {
//            idl.Types = baseIdl.Types;
//        }
//        else
//        {
//            idl.Types.AddRange(baseIdl.Types);
//        }
//    }
//    if (baseIdl.ClientToServerCalls != null)
//    {
//        if (idl.ClientToServerCalls == null)
//        {
//            idl.ClientToServerCalls = baseIdl.ClientToServerCalls;
//        }
//        else
//        {
//            idl.ClientToServerCalls.AddRange(baseIdl.ClientToServerCalls);
//        }
//    }
//    if (baseIdl.ServerToClientCalls != null)
//    {
//        if (idl.ServerToClientCalls == null)
//        {
//            idl.ServerToClientCalls = baseIdl.ServerToClientCalls;
//        }
//        else
//        {
//            idl.ServerToClientCalls.AddRange(baseIdl.ServerToClientCalls);
//        }
//    }
//}

StreamWriter libStream = new(File.Create("lib.rs"));
StreamWriter typesStream = null;

if (idl.Types != null)
{
    Console.WriteLine(idl.Types.Count + " type(s)");

    foreach (var type in idl.Types)
    {
        Console.WriteLine("  Emitting type " + type.Name);
        if (typesStream == null)
        {
            typesStream = new StreamWriter(File.Create("types.rs"));
        }
        TypeEmitter.Emit(typesStream, idl, type);
    }
    if (typesStream != null) libStream.WriteLine("pub mod types;");
}

int ipcNumber = 1;
StreamWriter ipcStream = null;
StreamWriter clientCallsStream = null;
StreamWriter clientHandlersStream = null;
StreamWriter serverCallsStream = null;
StreamWriter serverHandlersStream = null;

if (idl.ClientToServerCalls != null)
{
    Console.WriteLine(idl.ClientToServerCalls.Count + " client->server call(s)");

    foreach (var call in idl.ClientToServerCalls)
    {
        Console.WriteLine("  Emitting client->server call " + call.Name);
        if (ipcStream == null)
        {
            ipcStream = new StreamWriter(File.Create("ipc.rs"));
        }
        if (clientCallsStream == null)
        {
            clientCallsStream = new StreamWriter(File.Create("client_calls.rs"));
            clientCallsStream.WriteLine("extern crate chaos;");
            clientCallsStream.WriteLine("use chaos::channel::Channel;");
            clientCallsStream.WriteLine("use crate::types::*;");
            clientCallsStream.WriteLine("use crate::ipc::*;");
            clientCallsStream.WriteLine();
        }
        if (serverHandlersStream == null)
        {
            serverHandlersStream = new StreamWriter(File.Create("server_handlers.rs"));
            serverHandlersStream.WriteLine("extern crate chaos;");
        }
        var callName = CasedString.FromPascal(call.Name);
        ipcStream.WriteLine("pub const " + idl.Interface.Name.ToUpper() + "_" + callName.ToScreamingSnake() + "_CLIENT_MESSAGE: u64 = " + ipcNumber++ + ";");
        CallEmitter.Emit(clientCallsStream, idl, call);
        HandlerEmitter.Emit(serverHandlersStream, idl, call);
    }
    if (clientCallsStream != null)
    {
        libStream.WriteLine("pub mod client_calls;");
    }
}

if (idl.ServerToClientCalls != null)
{
    Console.WriteLine(idl.ServerToClientCalls.Count + " server->client call(s)");

    foreach (var call in idl.ServerToClientCalls)
    {
        Console.WriteLine("  Emitting server->client call " + call.Name);
        if (ipcStream == null)
        {
            ipcStream = new StreamWriter(File.Create("ipc.rs"));
        }
        if (serverCallsStream == null)
        {
            serverCallsStream = new StreamWriter(File.Create("server_calls.rs"));
            serverCallsStream.WriteLine("extern crate chaos;");
            serverCallsStream.WriteLine("use chaos::channel::Channel;");
            serverCallsStream.WriteLine("use crate::types::*;");
            serverCallsStream.WriteLine("use crate::ipc::*;");
            serverCallsStream.WriteLine();
        }
        if (clientHandlersStream == null)
        {
            clientHandlersStream = new StreamWriter(File.Create("client_handlers.rs"));
            clientHandlersStream.WriteLine("extern crate chaos;");
        }
        var callName = CasedString.FromPascal(call.Name);
        ipcStream.WriteLine("pub const " + idl.Interface.Name.ToUpper() + "_" + callName.ToScreamingSnake() + "_SERVER_MESSAGE: u64 = " + ipcNumber++ + ";");
        CallEmitter.Emit(serverCallsStream, idl, call);
        HandlerEmitter.Emit(clientHandlersStream, idl, call);
    }
    if (clientCallsStream != null)
    {
        libStream.WriteLine("pub mod client_calls;");
    }
}

if (idl.ClientToServerCalls != null || idl.ServerToClientCalls != null)
{
    libStream.WriteLine("mod ipc;");
}
else
{
    Console.WriteLine("Warning: No calls emitted!");
}

ipcStream?.Close();
libStream?.Close();
typesStream?.Close();
clientCallsStream?.Close();
clientHandlersStream?.Close();
serverCallsStream?.Close();
serverHandlersStream?.Close();

Console.WriteLine("Done");
