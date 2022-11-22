using System;
using System.Collections.Generic;
using System.Diagnostics.Tracing;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal class CallGenerator
    {
        private static IDLField GetFieldWithPath(IDLField field, string path)
        {
            field = field.Clone();
            field.Name = path == "" ? field.Name : path + "_" + field.Name;
            return field;
        }

        private static List<IDLField> GetFields(IDLField field, string path)
        {
            if (field.IsArray) return new List<IDLField> { GetFieldWithPath(field, path) };

            switch (field.Type)
            {
                case IDLField.FieldType.None:
                    throw new ArgumentException("Field of type None not accepted as parameter or return value to call");

                case IDLField.FieldType.U8:
                case IDLField.FieldType.U64:
                case IDLField.FieldType.I64:
                case IDLField.FieldType.Bool:
                case IDLField.FieldType.String:
                case IDLField.FieldType.Enum:
                case IDLField.FieldType.OneOfType:
                    return new List<IDLField> { GetFieldWithPath(field, path) };

                case IDLField.FieldType.CustomType:
                    return field.CustomType.Fields.Values.SelectMany(f => GetFields(f, path == "" ? field.Name : path + "_" + field.Name)).ToList();

                default:
                    throw new ArgumentException("Unknown type");
            }
        }

        private static List<IDLField> FlattenFields(List<IDLField> fields)
        {
            var result = new List<IDLField>();

            foreach (var field in fields)
            {
                result.AddRange(GetFields(field, ""));
            }

            return result;
        }

        private static bool IsFixedSize(IDLField.OneOfOption option)
        {
            return IsFixedSize(false, option.Type, null, option.CustomType, false);
        }

        private static bool IsFixedSize(IDLField field, bool checkArray)
        {
            return IsFixedSize(field.IsArray, field.Type, field.CustomOneOfOptions, field.CustomType, checkArray);
        }

        private static bool IsFixedSize(bool isArray, IDLField.FieldType type, List<IDLField.OneOfOption> oneOfOptions, IDLType customType, bool checkArray)
        {
            if (checkArray && isArray) return false;

            switch (type)
            {
                case IDLField.FieldType.None:
                    throw new ArgumentException("Field of type None not accepted as parameter or return value to call");

                case IDLField.FieldType.U8:
                case IDLField.FieldType.U64:
                case IDLField.FieldType.I64:
                case IDLField.FieldType.Bool:
                case IDLField.FieldType.Enum:
                    return true;

                case IDLField.FieldType.String:
                    return false;

                case IDLField.FieldType.OneOfType:
                    return oneOfOptions.All(IsFixedSize);

                case IDLField.FieldType.CustomType:
                    return customType.Fields.Values.All(f => IsFixedSize(f, true));

                default:
                    throw new ArgumentException("Unknown type");
            }
        }

        //private static bool IncludeInArgument(IDLField field)
        //{
        //    if (field.IsArray) return true;

        //    switch (field.Type)
        //    {
        //        case IDLField.FieldType.None:
        //            throw new ArgumentException("Field of type None not accepted as parameter or return value to call");

        //        case IDLField.FieldType.U8:
        //        case IDLField.FieldType.U64:
        //        case IDLField.FieldType.I64:
        //        case IDLField.FieldType.Bool:
        //        case IDLField.FieldType.String:
        //        case IDLField.FieldType.Enum:
        //            return true;

        //        case IDLField.FieldType.OneOfType:
        //            return false;

        //        case IDLField.FieldType.CustomType:
        //            return true;

        //        default:
        //            throw new ArgumentException("Unknown type");
        //    }
        //}

        private static bool IsArrayWithFixedSizeItems(IDLField field)
        {
            if (!field.IsArray) return false;
            return IsFixedSize(field, false);
        }

        private static string ToArgument(IDLField field, string owningTypeName)
        {
            if (IsArrayWithFixedSizeItems(field))
                return $"{field.Name}_count: usize";
            else
                return $"{field.Name}: {field.GetRustType(owningTypeName, true)}";
        }

        private static void AppendTypeWrite(SourceGenerator.SourceBlock block, IDLField field, List<string> sizeParts, List<string> returnVecs, string path)
        {
            var fieldPathUnderscore = path == "" ? field.Name : path + "_" + field.Name;
            var fieldPathAccessor = fieldPathUnderscore.Replace("_", ".");

            if (field.IsArray)
            {
                var typeName = field.GetInnerRustType("", false);
                if (IsArrayWithFixedSizeItems(field))
                {
                    block.AddLine($"*(pointer as *mut usize) = {fieldPathUnderscore}_count;");
                    block.AddLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                    block.AddLine($"let {fieldPathUnderscore} = Vec::<{typeName}>::from_raw_parts(pointer as *mut {typeName}, {fieldPathUnderscore}_count, {fieldPathUnderscore}_count);");
                    sizeParts.Add($"mem::size_of::<usize>() + {fieldPathUnderscore}_count * mem::size_of::<{typeName}>()");
                    returnVecs.Add($"ManuallyDrop::new({fieldPathUnderscore})");
                }
                else
                {
                    block.AddLine($"*(pointer as *mut usize) = {fieldPathUnderscore}.len();");
                    block.AddLine($"let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                    block.AddLine($"let mut _{fieldPathUnderscore}_size: usize = 0;");
                    var forBlock = block.AddBlock($"for item in {fieldPathUnderscore}.iter()");
                    forBlock.AddLine("let item_size = item.create_at_address(pointer);");
                    forBlock.AddLine("let pointer = pointer.offset(item_size as isize);");
                    forBlock.AddLine($"_{fieldPathUnderscore}_size += item_size;");
                    sizeParts.Add($"_{fieldPathUnderscore}_size");
                }
            }
            else
            {
                switch (field.Type)
                {
                    case IDLField.FieldType.None:
                        throw new ArgumentException("Unallowed field type None");

                    case IDLField.FieldType.U8:
                    case IDLField.FieldType.U64:
                    case IDLField.FieldType.I64:
                    case IDLField.FieldType.Bool:
                        block.AddLine($"(*object).{fieldPathAccessor} = {fieldPathUnderscore};");
                        break;

                    case IDLField.FieldType.String:
                        block.AddLine($"let _{field.Name}_length = {field.Name}.len();");
                        block.AddLine($"*(pointer as *mut usize) = _{field.Name}_length;");
                        block.AddLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                        block.AddLine($"core::ptr::copy({field.Name}.as_ptr(), pointer, _{field.Name}_length);");
                        block.AddLine($"let pointer = pointer.offset(_{field.Name}_length as isize);");

                        sizeParts.Add($"mem::size_of::<usize>() + _{field.Name}_length");
                        break;

                    case IDLField.FieldType.CustomType:
                        foreach (var typeField in field.CustomType.Fields.Values)
                        {
                            AppendTypeWrite(block, typeField, sizeParts, returnVecs, path == "" ? field.Name : path + "_" + field.Name);
                        }
                        break;
                }
            }
        }

        private static void GenerateWrite(SourceGenerator.SourceBlock block, string implName, List<IDLField> fields)
        {
            var flattenedFields = FlattenFields(fields);
            var functionArguments = flattenedFields.Select(f => ToArgument(f, implName));
            var arguments = string.Join(", ", functionArguments);

            var returns = new List<string>() { "usize" };
            foreach (var field in flattenedFields.Where(f => IsArrayWithFixedSizeItems(f)))
            {
                returns.Add($"ManuallyDrop<{field.GetRustType("", false)}>");
            }
            var returnTuple = returns.Count == 1 ? returns[0] : $"({string.Join(", ", returns)})";

            var body = block.AddBlock($"pub unsafe fn create_at_address(pointer: *mut u8, {arguments}) -> {returnTuple}");
            body.AddLine($"let object: *mut {implName} = mem::transmute(pointer);");
            body.AddLine($"let pointer = pointer.offset(mem::size_of::<{implName}>() as isize);");

            var sizeParts = new List<string>();
            sizeParts.Add($"mem::size_of::<{implName}>()");

            var returnVecs = new List<string>();
            foreach (var field in fields)
            {
                body.AddBlank();
                body.AddLine($"// {field.Name}");
                AppendTypeWrite(body, field, sizeParts, returnVecs, "");
            }

            // return
            var sizeString = string.Join(" + ", sizeParts);
            body.AddBlank();
            body.AddLine("// return");
            if (returns.Count == 1)
                body.AddLine(sizeString);
            else
                body.AddLine($"({sizeString}, {string.Join(", ", returnVecs)})");
        }

        private static void GenerateRead(SourceGenerator.SourceBlock block, List<IDLField> fields)
        {

        }

        internal static void GenerateCall(FileStream output, IDLCall call, int message_id)
        {
            var callName = CasedString.FromSnake(call.Name);

            if (call.Parameters.Count > 0)
            {
                var structName = $"{callName.ToPascal()}Parameters";

                var type = new IDLType
                {
                    Name = structName,
                    Fields = call.Parameters
                };
                TypeGenerator.GenerateType(output, type);

                var source = new SourceGenerator();
                var block = source.AddBlock($"impl {structName}");
                GenerateWrite(block, structName, call.Parameters.Values.ToList());
                GenerateRead(block, call.Parameters.Values.ToList());

                using (var writer = new StreamWriter(output, leaveOpen: true))
                {
                    writer.WriteLine(source.GetSource());
                    writer.WriteLine();
                }
            }

            if (call.ReturnValues.Count > 0)
            {
                var structName = $"{callName.ToPascal()}Returns";

                var type = new IDLType
                {
                    Name = structName,
                    Fields = call.ReturnValues
                };
                TypeGenerator.GenerateType(output, type);

                var source = new SourceGenerator();
                var block = source.AddBlock($"impl {structName}");
                GenerateWrite(block, structName, call.ReturnValues.Values.ToList());
                GenerateRead(block, call.ReturnValues.Values.ToList());

                using (var writer = new StreamWriter(output, leaveOpen: true))
                {
                    writer.WriteLine(source.GetSource());
                    writer.WriteLine();
                }
            }
        }
    }
}
