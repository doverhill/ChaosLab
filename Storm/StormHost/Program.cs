using Storm;

List<StartupCommand> startupList = new List<StartupCommand>();

if (File.Exists("startup.list"))
{
    startupList = File
        .ReadAllLines("startup.list")
        .Select(l => l.Split(' ', StringSplitOptions.RemoveEmptyEntries))
        .Where(p => p.Length == 3 && !p[0].StartsWith("#"))
        .Select(parts => new StartupCommand(int.Parse(parts[0]), parts[1], parts[2]))
        .ToList();
}

var kernel = new Kernel();
kernel.Start(startupList);
