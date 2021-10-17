using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal class CallEmitter
    {
        public static void Emit(IDL idl, IDLCall call)
        {
            using var file = File.Create(call.Name + "Call.cs");
            using var writer = new StreamWriter(file);
            var emitter = new CommonEmitter(writer);

            emitter.FileIntro(idl.Interface.Name + ".IPC");
            emitter.WriteCall(idl, call);
            emitter.FileOutro();
        }
    }
}
