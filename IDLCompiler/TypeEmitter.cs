using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal class TypeEmitter
    {
        public static void Emit(IDL idl, IDLType type)
        {
            using var file = File.Create(type.Name + ".cs");
            using var writer = new StreamWriter(file);
            var emitter = new CommonEmitter(writer);

            emitter.FileIntro(idl.Interface.Name + ".IPC");
            emitter.WriteStruct(idl, type.Name, type.Fields, -1);
            emitter.FileOutro();
        }
    }
}
