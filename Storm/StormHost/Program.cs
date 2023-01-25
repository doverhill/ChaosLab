using Storm;

List<StartupCommand> startupList = new List<StartupCommand>();

if (File.Exists("startup.list"))
{
    startupList = File.ReadAllLines("startup.list").Select(l =>
    {
        var parts = l.Split(' ');
        return new StartupCommand(int.Parse(parts[0]), parts[1]);
    }).ToList();
}

var kernel = new Kernel();
kernel.Start(startupList);
