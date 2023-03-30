using Storm;
using System.Collections.Generic;

var startupList = new List<StartupCommand> {
    new StartupCommand("server_root", "Server/Root", "target/debug/server_root.exe", null,
        new List<string> { "*.*:*" },
        new List<string> { "*.*:*" },
        0
    )
};

//if (File.Exists("startup.list"))
//{
//    startupList = File
//        .ReadAllLines("startup.list")
//        .Select(l => l.Split(' ', StringSplitOptions.RemoveEmptyEntries))
//        .Where(p => p.Length == 3 && !p[0].StartsWith("#"))
//        .Select(parts => new StartupCommand(int.Parse(parts[0]), parts[1], parts[2]))
//        .ToList();
//}



var kernel = new Kernel();
kernel.Start(startupList);
