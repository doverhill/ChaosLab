namespace IDLCompiler
{
    internal class CommonEmitter
    {
        private StreamWriter writer;
        private TypeEmitter typeEmitter;
        private IDL idl;
        private int indent = 0;
        private const int IndentationSteps = 4;

        public CommonEmitter(IDL idl, StreamWriter writer, TypeEmitter typeEmitter)
        {
            this.idl = idl;
            this.writer = writer;
            this.typeEmitter = typeEmitter;
        }

        private void WriteIndent()
        {
            writer.Write(new string(' ', IndentationSteps * indent));
        }

        private void WriteField(Field field, bool lastField)
        {
            WriteIndent();
            writer.Write(field.Name.ToSnake() + ": ");
            writer.Write(field.GetStructType());

            if (lastField)
            {
                writer.WriteLine();
            }
            else
            {
                writer.WriteLine(",");
            }
        }

        public void WriteStruct(CasedString name, List<Field> fields)
        {
            // write struct
            WriteIndent(); writer.WriteLine("#[allow(dead_code)]");
            WriteIndent(); writer.WriteLine("pub struct " + name.ToPascal() + " {"); indent++;

            for (var fieldIndex = 0; fieldIndex < fields.Count; fieldIndex++)
            {
                var field = fields[fieldIndex];
                var lastField = fieldIndex == fields.Count - 1;
                WriteField(field, lastField);
            }

            indent--; WriteIndent(); writer.WriteLine("}");
            writer.WriteLine();
        }

        private string GetConstructorParameter(Field field)
        {
            return field.Name.ToSnake() + ": " + field.GetConstructorType();
        }

        private void WriteConstructorParameters(List<Field> fields)
        {
            writer.Write(string.Join(", ", fields.Select(f => GetConstructorParameter(f))));
        }

        private void WriteConstructorAssignment(Field field, bool lastField)
        {
            WriteIndent();
            writer.Write(field.Name.ToSnake() + ": ");
            if (field.Type == Field.DataType.String)
            {
                writer.Write("[0u8; 44]");
            }
            else
            {
                writer.Write(field.Name.ToSnake());
            }

            if (lastField)
            {
                writer.WriteLine();
            }
            else
            {
                writer.WriteLine(",");
            }
        }

        public void WriteImplementation(CasedString name, List<Field> fields)
        {
            // write struct
            WriteIndent(); writer.WriteLine("#[allow(dead_code)]");
            WriteIndent(); writer.WriteLine("impl " + name.ToPascal() + " {"); indent++;

            // constructor
            WriteIndent(); writer.Write("pub fn new("); WriteConstructorParameters(fields); writer.WriteLine(") -> " + name.ToPascal() + " {"); indent++;
            WriteIndent(); writer.WriteLine("let constructed_" + name.ToSnake() + " = " + name.ToPascal() + " {"); indent++;
            for (var fieldIndex = 0; fieldIndex < fields.Count; fieldIndex++)
            {
                var field = fields[fieldIndex];
                var lastField = fieldIndex == fields.Count - 1;
                WriteConstructorAssignment(field, lastField);
            }
            indent--; WriteIndent(); writer.WriteLine("};");

            foreach (var field in fields)
            {
                if (field.Type == Field.DataType.String)
                {
                    WriteIndent(); writer.WriteLine("unsafe { core::ptr::copy(" + field.Name.ToSnake() + ".as_ptr(), core::ptr::addr_of!(constructed_" + name.ToSnake() + "." + field.Name.ToSnake() + ") as *mut u8, core::cmp::min(98, " + field.Name.ToSnake() + ".len())); }");
                }
            }

            WriteIndent(); writer.WriteLine("constructed_" + name.ToSnake());
            indent--; WriteIndent(); writer.WriteLine("}");

            // getters for strings
            foreach (var field in fields)
            {
                if (field.Type == Field.DataType.String)
                {
                    writer.WriteLine();
                    WriteIndent(); writer.WriteLine("pub fn get_" + field.Name.ToSnake() + "(&self) -> &str {"); indent++;
                    WriteIndent(); writer.WriteLine("unsafe { core::str::from_utf8_unchecked(&self." + field.Name.ToSnake() + ") }");
                    indent--; WriteIndent(); writer.WriteLine("}");
                }
            }

            indent--; WriteIndent(); writer.WriteLine("}");
            writer.WriteLine();
        }

        private string GetParameterString(string parameter)
        {
            var field = new Field(parameter, idl.Types);
            return field.Name.ToSnake() + ": " + field.GetConstructorType();
        }

        private string GetParametersString(IDLCall call)
        {
            if (call.Parameters != null && call.Parameters.Count > 0)
            {
                return string.Join(", ", call.Parameters.Select(p => GetParameterString(p)));
            }

            return "";
        }

        //private string GetReturnString(string ret)
        //{
        //    var (fieldType, fieldName, isList, listCount) = ParseField(ret);
        //    return fieldType;
        //}

        //private string GetReturnsString(IDLCall call)
        //{
        //    if (call.Returns != null && call.Returns.Count > 0)
        //    {
        //        if (call.Returns.Count == 1)
        //        {
        //            return " -> " + GetReturnString(call.Returns[0]);
        //        }
        //        else
        //        {
        //            return " -> (" + string.Join(", ", call.Returns.Select(r => GetReturnString(r))) + ")";
        //        }
        //    }
        //    return "";
        //}

        public void WriteCall(IDLCall call)
        {
            var callName = CasedString.FromPascal(call.Name);

            // generate safe call, copies struct
            WriteIndent(); writer.WriteLine("#[allow(dead_code)]");
            WriteIndent(); writer.WriteLine("pub fn " + idl.Interface.Name + "_" + callName.ToSnake() + "(channel: Channel, " + GetParametersString(call) + ")" +  " {"); indent++;

            indent--; WriteIndent(); writer.WriteLine("}");
            writer.WriteLine();

            // generate unsafe faster call, returns pointer to struct
            WriteIndent(); writer.WriteLine("#[allow(dead_code)]");
            WriteIndent(); writer.WriteLine("unsafe fn " + idl.Interface.Name + "_" + callName.ToSnake() + "_raw(channel: Channel, " + GetParametersString(call) + ") -> ptr {"); indent++;

            indent--; WriteIndent(); writer.WriteLine("}");
            writer.WriteLine();
        }
    }
}
