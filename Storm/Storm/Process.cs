using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Storm
{
    internal class Process
    {
        public ulong PID;
        public string Name;

        public Process(ulong pID, string name)
        {
            PID = pID;
            Name = name;
        }
    }
}
