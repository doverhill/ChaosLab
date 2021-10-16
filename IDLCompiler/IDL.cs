using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal class IDLInterface
    {
        public string? Name;
        public int Version;
    }

    internal class IDLType
    {
        public string? Name;
        public List<string>? Fields;
    }

    internal class IDLCall
    {
        public string? Name;
        public List<string>? Parameters;
        public List<string>? Returns;
        public int BatchSize = 64;
    }

    internal class IDL
    {
        public IDLInterface? Interface;
        public List<IDLType>? Types;
        public List<IDLCall>? InboundCalls;
        public List<IDLCall>? OutboundCalls;
    }
}
