using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal class TypeEmitter
    {
        public static void Emit(StreamWriter writer, IDL idl, IDLType type)
        {
            var emitter = new CommonEmitter(idl, writer);
            emitter.WriteStruct(type.Name, type.Fields, -1);
        }
    }
}
