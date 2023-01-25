using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Storm
{
    public class StartupCommand
    {
        public int DelayMs;
        public string Path;
        public string Executable;

        public StartupCommand(int delayMs, string path, string executable)
        {
            DelayMs = delayMs;
            Path = path;
            Executable = executable;
        }
    }
}
