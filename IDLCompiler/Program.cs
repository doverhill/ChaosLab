using IDLCompiler;
using System.Text.Json;

if (args.Length < 1)
{
    Console.WriteLine("Error: Use with <interface.idl>");
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

Console.WriteLine(idl.Types?.Count + " types");
if (idl.Types != null)
{
    foreach (var type in idl.Types)
    {
        Console.WriteLine("  Emitting type " + type.Name);
        TypeEmitter.Emit(idl, type);
    }
}

Console.WriteLine(idl.InboundCalls?.Count + " inbound calls");
if (idl.InboundCalls != null)
{
    foreach (var call in idl.InboundCalls)
    {
        Console.WriteLine("  Emitting call " + call.Name);
        CallEmitter.Emit(idl, call);
    }
}

Console.WriteLine(idl.OutboundCalls?.Count + " outbound calls");
if (idl.OutboundCalls != null)
{
    foreach (var call in idl.OutboundCalls)
    {
        Console.WriteLine("  Emitting call " + call.Name);
        CallEmitter.Emit(idl, call);
    }
}

Console.WriteLine("Done");