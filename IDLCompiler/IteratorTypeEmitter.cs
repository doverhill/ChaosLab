using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal class IteratorTypeEmitter
    {
        public static void Emit(IDL idl, CasedString callName, Field field)
        {
            var stream = new StreamWriter(File.Create("types/" + callName.ToSnake() + "_" + field.TypeName.ToSnake() + "_iterator.rs"));
            var output = new StructuredWriter(stream);
            Emit(output, idl, callName, field.TypeName.ToPascal(), false, CallEmitter.CallType.NotRelevant, field.TypeName.ToScreamingSnake(), true);
            stream.Close();

            // append to crate
            File.AppendAllLines("types/mod.rs", new string[]
            {
                "mod " + callName.ToSnake() + "_" + field.TypeName.ToSnake() + "_iterator;",
                "pub use " + callName.ToSnake() + "_" + field.TypeName.ToSnake() + "_iterator::" + callName.ToPascal() + field.TypeName.ToPascal() + "Iterator;",
                ""
            });
        }

        public static void EmitMixed(IDL idl, CasedString callName, List<Field> fields, CallEmitter.CallType callType)
        {
            var protocolName = CasedString.FromPascal(idl.Interface.Name);
            var typeName = callName.ToSnake() + "_mixed_" + callType.ToString().ToLower() + "_iterator";
            var stream = new StreamWriter(File.Create("types/" + typeName + ".rs"));
            var output = new StructuredWriter(stream);
            EmitEnum(output, idl, callName, callType, fields);
            Emit(output, idl, callName, callName.ToPascal() + callType.ToString() + "Enum", true, callType, callName.ToScreamingSnake() + "_" + callType.ToString().ToUpper() + "_ENUM", false);
            stream.Close();

            // append to crate
            File.AppendAllLines("types/mod.rs", new string[]
            {
                
                "mod " + typeName + ";",
                "pub use " + typeName + "::" + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + "_" + callType.ToString().ToUpper() + "_ENUM_OBJECT_ID;",
                "pub use " + typeName + "::" + callName.ToPascal() + callType.ToString() + "Enum;",
                "pub use " + typeName + "::" + callName.ToPascal() + "Mixed" + callType.ToString() + "Iterator;",
                ""
            });
        }

        public static void EmitEnum(StructuredWriter output, IDL idl, CasedString callName, CallEmitter.CallType callType, List<Field> fields)
        {
            var protocolName = CasedString.FromPascal(idl.Interface.Name);

            output.WriteLine("use library_chaos::{ Channel, ChannelObject };");
            output.WriteLine("use core::{ mem, ptr, str, slice };");
            output.WriteLine("use std::iter::Iterator;");
            output.WriteLine("use std::sync::Arc;");
            output.WriteLine("use std::sync::Mutex;");
            output.BlankLine();

            output.WriteLine("pub const " + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + "_" + callType.ToString().ToUpper() + "_ENUM_OBJECT_ID: usize = " + TypeEmitter.GetId() + ";");
            output.BlankLine();

            output.WriteLine("pub enum " + callName.ToPascal() + callType.ToString() + "Enum", true);
            Common.ForEach(fields, (field, isLast) =>
            {
                output.WriteLine(field.TypeName.ToPascal() + "(crate::" + field.TypeName.ToPascal() + ")" + (isLast ? "" : ","));
            });
            output.CloseScope();
            output.BlankLine();

            // impl ChannelObject
            output.WriteLine("impl ChannelObject for " + callName.ToPascal() + callType.ToString() + "Enum", true);

            // write_to_channel
            output.WriteLine("unsafe fn write_to_channel(self, pointer: *mut u8) -> usize", true);
            output.WriteLine("match self", true);
            Common.ForEach(fields, (field, isLast) =>
            {
                output.WriteLine("Self::" + field.TypeName.ToPascal() + "(object) =>", true);
                output.WriteLine("*(pointer as *mut usize) = crate::" + protocolName.ToScreamingSnake() + "_" + field.TypeName.ToScreamingSnake() + "_OBJECT_ID;");
                output.WriteLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                output.WriteLine("let size = object.write_to_channel(pointer);");
                output.WriteLine("mem::size_of::<usize>() + size");
                output.CloseScope(isLast ? "" : ",");
            });
            output.CloseScope();
            output.CloseScope();
            output.BlankLine();

            // from_channel
            output.WriteLine("unsafe fn from_channel(pointer: *const u8) -> Self", true);
            output.WriteLine("let kind = *(pointer as *const usize);");
            output.WriteLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
            output.BlankLine();
            output.WriteLine("match kind", true);
            Common.ForEach(fields, (field, isLast) =>
            {
                output.WriteLine("crate::" + protocolName.ToScreamingSnake() + "_" + field.TypeName.ToScreamingSnake() + "_OBJECT_ID =>", true);
                output.WriteLine("Self::" + field.TypeName.ToPascal() + "(crate::" + field.TypeName.ToPascal() + "::from_channel(pointer))");
                output.CloseScope(",");
            });
            output.WriteLine("_ =>", true);
            output.WriteLine("panic!(\"Received unexpected value for " + callName.ToPascal() + callType.ToString() + "Enum\");");
            output.CloseScope();
            output.CloseScope();
            output.CloseScope();

            output.CloseScope();
            output.BlankLine();
        }

        public static void Emit(StructuredWriter output, IDL idl, CasedString callName, string pascalTypeName, bool isMixed, CallEmitter.CallType callType, string screamingSnakeTypeName, bool emitImports)
        {
            var protocolName = CasedString.FromPascal(idl.Interface.Name);

            if (emitImports)
            {
                output.WriteLine("use library_chaos::Channel;");
                output.WriteLine("use core::{ mem, ptr, str, slice };");
                output.WriteLine("use std::iter::Iterator;");
                output.WriteLine("use std::sync::Arc;");
                output.WriteLine("use std::sync::Mutex;");
                output.BlankLine();
            }

            // write type struct
            output.WriteLine("pub struct " + callName.ToPascal() + (isMixed ? ("Mixed" + callType.ToString()) : pascalTypeName) + "Iterator", true);
            output.WriteLine("channel_reference: Arc<Mutex<Channel>>,");
            output.WriteLine("index: usize,");
            output.WriteLine("item_count: usize");
            output.CloseScope();
            output.BlankLine();

            // impl
            output.WriteLine("impl " + callName.ToPascal() + (isMixed ? ("Mixed" + callType.ToString()) : pascalTypeName) + "Iterator", true);
            output.WriteLine("pub fn new(channel_reference: Arc<Mutex<Channel>>) -> Self", true);
            output.WriteLine("let channel = channel_reference.lock().unwrap();");
            output.WriteLine("let item_count = channel.get_object_count();");
            output.WriteLine("drop(channel);");
            output.BlankLine();
            output.WriteLine(callName.ToPascal() + (isMixed ? ("Mixed" + callType.ToString()) : pascalTypeName) + "Iterator", true);
            output.WriteLine("channel_reference: channel_reference.clone(),");
            output.WriteLine("index: 0,");
            output.WriteLine("item_count: item_count");
            output.CloseScope();
            output.CloseScope();
            output.CloseScope();
            output.BlankLine();

            // impl Iterator
            output.WriteLine("impl Iterator for " + callName.ToPascal() + (isMixed ? ("Mixed" + callType.ToString()) : pascalTypeName) + "Iterator", true);
            output.WriteLine("type Item = crate::" + pascalTypeName + ";");
            output.BlankLine();
            output.WriteLine("fn next(&mut self) -> Option<Self::Item>", true);
            output.WriteLine("if self.index < self.item_count", true);
            output.WriteLine("let channel = self.channel_reference.lock().unwrap();");
            output.WriteLine("self.index += 1;");
            output.WriteLine("match channel.get_object::<crate::" + pascalTypeName + ">(self.index - 1, crate::" + protocolName.ToScreamingSnake() + "_" + screamingSnakeTypeName + "_OBJECT_ID)", true);
            output.WriteLine("Ok(object) =>", true);
            output.WriteLine("Some(object)");
            output.CloseScope(",");
            output.WriteLine("Err(_) =>", true);
            output.WriteLine("None");
            output.CloseScope();
            output.CloseScope();
            output.CloseScope();
            output.WriteLine("else", true);
            output.WriteLine("None");
            output.CloseScope();
            output.CloseScope();
            output.CloseScope();
            output.BlankLine();
        }
    }
}
