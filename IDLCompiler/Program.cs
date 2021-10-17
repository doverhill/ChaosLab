using IDLCompiler;
using System.Text.Json;

if (args.Length < 2)
{
    Console.WriteLine("Error: Use with <client|server> <interface.idl>");
    return;
}

var side = args[0];
var filename = args[1];
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

Console.WriteLine(idl.Types?.Count + " types");
if (idl.Types != null)
{
    foreach (var type in idl.Types)
    {
        Console.WriteLine("  Emitting type " + type.Name);
        TypeEmitter.Emit(idl, type);
    }
}

if (side == "client")
{
    Console.WriteLine(idl.ClientToServerCalls?.Count + " client->server calls");
    if (idl.ClientToServerCalls != null)
    {
        foreach (var call in idl.ClientToServerCalls)
        {
            Console.WriteLine("  Emitting call " + call.Name);
            CallEmitter.Emit(idl, call);
        }
    }

    Console.WriteLine(idl.ServerToClientCalls?.Count + " server->client handlers");
    if (idl.ServerToClientCalls != null)
    {
        foreach (var call in idl.ServerToClientCalls)
        {
            Console.WriteLine("  Emitting handler " + call.Name);
            HandlerEmitter.Emit(idl, call);
        }
    }
}
else if (side == "server")
{
    Console.WriteLine(idl.ServerToClientCalls?.Count + " server->client calls");
    if (idl.ServerToClientCalls != null)
    {
        foreach (var call in idl.ServerToClientCalls)
        {
            Console.WriteLine("  Emitting call " + call.Name);
            CallEmitter.Emit(idl, call);
        }
    }

    Console.WriteLine(idl.ClientToServerCalls?.Count + " client->server handlers");
    if (idl.ClientToServerCalls != null)
    {
        foreach (var call in idl.ClientToServerCalls)
        {
            Console.WriteLine("  Emitting handler " + call.Name);
            HandlerEmitter.Emit(idl, call);
        }
    }
}
else
{
    Console.WriteLine("Error: Unknown side '" + side + "'");
}

Console.WriteLine("Done");