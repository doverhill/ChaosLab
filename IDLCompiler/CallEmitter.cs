using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal class CallEmitter
    {
        public static void Emit(StreamWriter writer, IDL idl, IDLCall call)
        {
            var emitter = new CommonEmitter(idl, writer);
            emitter.WriteCall(call);
        }
    }
}
