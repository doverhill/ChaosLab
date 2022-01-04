namespace IDLCompiler
{
    internal class TypeEmitter
    {
        private static int TypeId = 1;

        public static void Reset()
        {
            // clear all files in types/
            if (Directory.Exists("types")) Directory.Delete("types", true);
            Directory.CreateDirectory("types");
        }

        public static void Emit(IDL idl, IDLType type)
        {
            var protocolName = CasedString.FromPascal(idl.Interface.Name);
            var typeName = CasedString.FromPascal(type.Name);
            var stream = new StreamWriter(File.Create("types/" + typeName.ToSnake() + ".rs"));
            var output = new StructuredWriter(stream);
            Emit(output, idl, type, true);
            stream.Close();

            // append to
            File.AppendAllLines("types/mod.rs", new string[]
            {
                "mod " + typeName.ToSnake() + ";",
                "pub use " + typeName.ToSnake() + "::" + typeName.ToPascal() + ";",
                "pub use " + typeName.ToSnake() + "::" + protocolName.ToScreamingSnake() + "_" + typeName.ToScreamingSnake() + "_OBJECT_ID;",
                ""
            });
        }

        private static string GetFixedSize(List<Field> fields)
        {
            return string.Join(" + ", fields.Where(f => f.Type != Field.DataType.String).Select(f => "mem::size_of::<" + f.GetCallType() + ">()"));
        }

        private static string GetConstructorArguments(List<Field> fields)
        {
            return string.Join(", ", fields.Select(f => f.Name.ToSnake() + ": " + f.GetCallType()));
        }

        private static void WriteConstructorFields(StructuredWriter output, List<Field> fields)
        {
            Common.ForEach(fields, (field, isLast) =>
            {
                output.WriteLine(field.Name.ToSnake() + ": " + field.Name.ToSnake() + (field.Type == Field.DataType.String ? ".to_string()" : "") + (isLast ? "" : ","));
            });
        }

        public static void Emit(StructuredWriter output, IDL idl, IDLType type, bool emitImports)
        {
            if (type.Inherits != null)
            {
                var baseType = idl.Types.FirstOrDefault(t => t.Name == type.Inherits);
                if (type.Fields == null)
                {
                    type.Fields = baseType.Fields;
                }
                else
                {
                    type.Fields = baseType.Fields.Concat(type.Fields).ToList();
                }
            }
            if (type.Fields == null || type.Fields.Count == 0) throw new Exception("Type '" + type.Name + "' does not have any fields");

            var protocolName = CasedString.FromPascal(idl.Interface.Name);
            var typeName = CasedString.FromPascal(type.Name);
            var fields = type.Fields.Select(f => new Field(f, idl.Types)).ToList();
            var fixedFields = fields.Where(f => f.Type != Field.DataType.String).ToList();
            var dynamicFields = fields.Where(f => f.Type == Field.DataType.String).ToList();

            // write imports
            if (emitImports)
            {
                output.WriteLine("use library_chaos::ChannelObject;");
                output.WriteLine("use core::{ mem, ptr, str, slice };");
                output.BlankLine();
            }

            // writer object id
            output.WriteLine("pub const " + protocolName.ToScreamingSnake() + "_" + typeName.ToScreamingSnake() + "_OBJECT_ID: usize = " + TypeId++ + ";");
            output.BlankLine();

            // write type struct
            output.WriteLine("#[derive(Default)]");
            output.WriteLine("pub struct " + typeName.ToPascal() + " {");
            output.EnterBlock();
            output.WriteLine("// fixed size fields");
            Common.ForEach(fixedFields, (field, isLast) =>
            {
                output.WriteLine("pub " + field.Name.ToSnake() + ": " + field.GetStructType() + (isLast ? "" : ","));
            });
            output.WriteLine("// dynamically sized fields");
            Common.ForEach(dynamicFields, (field, isLast) =>
            {
                output.WriteLine("pub " + field.Name.ToSnake() + ": " + field.GetStructType() + (isLast ? "" : ","));
            });
            output.LeaveBlock();
            output.WriteLine("}");
            output.BlankLine();

            // impl
            output.WriteLine("impl " + typeName.ToPascal() + " {");
            output.EnterBlock();
            output.WriteLine("const FIXED_SIZE: usize = " + GetFixedSize(fields) + ";");
            output.BlankLine();

            // constructor
            output.WriteLine("pub fn new(" + GetConstructorArguments(fields) + ") -> Self {");
            output.EnterBlock();
            output.WriteLine(typeName.ToPascal() + " {");
            output.EnterBlock();
            WriteConstructorFields(output, fields);
            output.LeaveBlock();
            output.WriteLine("}");
            output.LeaveBlock();
            output.WriteLine("}");

            output.LeaveBlock();
            output.WriteLine("}");
            output.BlankLine();

            // impl ChannelObject
            output.WriteLine("impl ChannelObject for " + typeName.ToPascal() + " {");
            output.EnterBlock();

            // write_to_channel
            output.WriteLine("unsafe fn write_to_channel(self, pointer: *mut u8) -> usize {");
            output.EnterBlock();

            if (fixedFields.Count > 0)
            {
                output.WriteLine("// write fixed size fields");
                output.WriteLine("ptr::copy(mem::transmute::<&" + typeName.ToPascal() + ", *mut u8>(&self), pointer as *mut u8, Self::FIXED_SIZE);");
                if (dynamicFields.Count > 0) output.WriteLine("let pointer = pointer.offset(Self::FIXED_SIZE as isize);");
            }

            Common.ForEach(dynamicFields, (field, isLast) =>
            {
                output.BlankLine();
                output.WriteLine("// write dynamically sized field " + field.Name.ToSnake());
                output.WriteLine("let length = self." + field.Name.ToSnake() + ".len();");
                output.WriteLine("*(pointer as *mut usize) = len;");
                output.WriteLine("let pointer = pointer.offset(mem::size_of::<usize>());");
                output.WriteLine("ptr::copy(self." + field.Name.ToSnake() + ".as_ptr(), pointer, length);");
                if (!isLast)
                {
                    output.WriteLine("let pointer = pointer.offset(length as isize);");
                }
            });

            output.LeaveBlock();
            output.WriteLine("}");
            output.BlankLine();

            // from_channel
            output.WriteLine("unsafe fn from_channel(pointer: *mut u8) -> Self {");
            output.EnterBlock();

            output.WriteLine("let mut object = " + typeName.ToPascal() + "::default();");
            output.BlankLine();

            if (fixedFields.Count > 0)
            {
                output.WriteLine("// read fixed size fields");
                output.WriteLine("ptr::copy(pointer as *mut u8, mem::transmute::<&" + typeName.ToPascal() + ", *mut u8>(&object), Self::FIXED_SIZE);");
                if (dynamicFields.Count > 0) output.WriteLine("let pointer = pointer.offset(Self::FIXED_SIZE as isize);");
            }

            Common.ForEach(dynamicFields, (field, isLast) =>
            {
                output.BlankLine();
                output.WriteLine("// read dynamically sized field " + field.Name.ToSnake());
                output.WriteLine("let length = *(pointer as *const usize);");
                output.WriteLine("let pointer = pointer.offset(mem::size_of::<usize>());");
                output.WriteLine("object." + field.Name.ToSnake() + " = str::from_utf8_unchecked(slice::from_raw_parts(pointer as *const u8, length)).to_owned();");
                if (!isLast)
                {
                    output.WriteLine("let pointer = pointer.offset(length as isize);");
                }
            });

            output.LeaveBlock();
            output.WriteLine("}");

            output.LeaveBlock();
            output.WriteLine("}");
            output.BlankLine();
        }
    }
}
