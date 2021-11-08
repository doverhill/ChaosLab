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
            if (type.Fields == null || type.Fields.Count == 0) throw new Exception("Type '" + type.Name + "' does not have any fields");

            var emitter = new CommonEmitter(idl, writer);
            var typeName = CasedString.FromPascal(type.Name);
            var fields = type.Fields.Select(f => new Field(f, idl.Types)).ToList();
            emitter.WriteStruct(typeName, fields);
            emitter.WriteImplementation(typeName, fields);
        }
    }
}
