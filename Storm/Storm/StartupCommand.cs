using System.Collections.Generic;

namespace Storm {
    public class StartupCommand
    {
        public string Name;
        public string Path;
        public string Executable;
        public string Arguments;
        public List<string> Capabilities;
        public List<string> Grantables;
        public int DelayMilliseconds;

        public StartupCommand(string name, string path, string executable, string arguments, List<string> capabilities, List<string> grantables, int delayMilliseconds) {
            Name = name;
            Path = path;
            Executable = executable;
            Arguments = arguments;
            Capabilities = capabilities;
            Grantables = grantables;
            DelayMilliseconds = delayMilliseconds;
        }
    }
}
