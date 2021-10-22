using System.Reflection;

if (args.Length == 0)
{
    Console.WriteLine("Error: Provide process dll as argument");
    return;
}

var applicationName = args[0];
Console.WriteLine("Running " + applicationName);

try
{
    var assembly = Assembly.LoadFrom(applicationName);
    var type = assembly.GetType("Chaos.Root");
    var method = type.GetMethod("Entry");
    method.Invoke(null, null);
    Console.WriteLine("Successfully ran " + applicationName);
}
catch (Exception e)
{
    Console.WriteLine("Failed to run " + applicationName + ": " + e.Message);
    if (e.InnerException != null)
    {
        Console.WriteLine(e.InnerException.Message);
        Console.WriteLine(e.InnerException.StackTrace.ToString());
    }
}
