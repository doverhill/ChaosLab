using IDLCompiler;
using System.Text.Json;

if (args.Length < 1)
{
    Console.WriteLine("Error: Use with <idl.json>");
    return;
}

var filename = args[0];
var fileContents = File.ReadAllText(filename);
var options = new JsonSerializerOptions
{
    IncludeFields = true,
    ReadCommentHandling = JsonCommentHandling.Skip
};
var idl = JsonSerializer.Deserialize<IDL>(fileContents, options);

if (idl == null)
{
    Console.WriteLine("Error: Failed to read IDL file");
    return;
}

if (idl.Interface == null)
{
    Console.WriteLine("Error: IDL file is missing Interface description");
    return;
}

Console.WriteLine("Building interface " + idl.Interface.Name + " (version " + idl.Interface.Version + ")");

if (idl.Interface.InheritsFrom != null)
{
    Console.WriteLine("Inherits from " + idl.Interface.InheritsFrom);
    var path = Path.Combine(Path.GetDirectoryName(filename), idl.Interface.InheritsFrom);
    var baseContents = File.ReadAllText(path);
    var baseIdl = JsonSerializer.Deserialize<IDL>(baseContents, options);

    if (baseIdl.Types != null)
    {
        if (idl.Types == null)
        {
            idl.Types = baseIdl.Types;
        }
        else
        {
            idl.Types.AddRange(baseIdl.Types);
        }
    }
    if (baseIdl.ClientToServerCalls != null)
    {
        if (idl.ClientToServerCalls == null)
        {
            idl.ClientToServerCalls = baseIdl.ClientToServerCalls;
        }
        else
        {
            idl.ClientToServerCalls.AddRange(baseIdl.ClientToServerCalls);
        }
    }
    if (baseIdl.ServerToClientCalls != null)
    {
        if (idl.ServerToClientCalls == null)
        {
            idl.ServerToClientCalls = baseIdl.ServerToClientCalls;
        }
        else
        {
            idl.ServerToClientCalls.AddRange(baseIdl.ServerToClientCalls);
        }
    }
}

StreamWriter libStream = new StreamWriter(File.Create("lib.rs"));
StreamWriter typesStream = null;

Console.WriteLine(idl.Types?.Count + " types");
if (idl.Types != null)
{
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

StreamWriter clientCallsStream = null;
StreamWriter clientHandlersStream = null;
StreamWriter serverCallsStream = null;
StreamWriter serverHandlersStream = null;

Console.WriteLine(idl.ClientToServerCalls?.Count + " client->server calls");
if (idl.ClientToServerCalls != null)
{
    foreach (var call in idl.ClientToServerCalls)
    {
        Console.WriteLine("  Emitting client->server call " + call.Name);
        if (clientCallsStream == null)
        {
            clientCallsStream = new StreamWriter(File.Create("client_calls.rs"));
        }
        CallEmitter.Emit(clientCallsStream, idl, call);
    }
    if (clientCallsStream != null) libStream.WriteLine("pub mod client_calls;");
}


//if (side == "client")
//{
//    Console.WriteLine(idl.ClientToServerCalls?.Count + " client->server calls");
//    if (idl.ClientToServerCalls != null)
//    {
//        foreach (var call in idl.ClientToServerCalls)
//        {
//            Console.WriteLine("  Emitting call " + call.Name);
//            CallEmitter.Emit(idl, call);
//        }
//    }

//    Console.WriteLine(idl.ServerToClientCalls?.Count + " server->client handlers");
//    if (idl.ServerToClientCalls != null)
//    {
//        foreach (var call in idl.ServerToClientCalls)
//        {
//            Console.WriteLine("  Emitting handler " + call.Name);
//            HandlerEmitter.Emit(idl, call);
//        }
//    }
//}
//else if (side == "server")
//{
//    Console.WriteLine(idl.ServerToClientCalls?.Count + " server->client calls");
//    if (idl.ServerToClientCalls != null)
//    {
//        foreach (var call in idl.ServerToClientCalls)
//        {
//            Console.WriteLine("  Emitting call " + call.Name);
//            CallEmitter.Emit(idl, call);
//        }
//    }

//    Console.WriteLine(idl.ClientToServerCalls?.Count + " client->server handlers");
//    if (idl.ClientToServerCalls != null)
//    {
//        foreach (var call in idl.ClientToServerCalls)
//        {
//            Console.WriteLine("  Emitting handler " + call.Name);
//            HandlerEmitter.Emit(idl, call);
//        }
//    }
//}
//else
//{
//    Console.WriteLine("Error: Unknown side '" + side + "'");
//}

libStream?.Close();
typesStream?.Close();
clientCallsStream?.Close();
clientHandlersStream?.Close();
serverCallsStream?.Close();
serverHandlersStream?.Close();

Console.WriteLine("Done");