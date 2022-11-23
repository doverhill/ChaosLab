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

        public static void AppendTypeWrite(SourceGenerator.SourceBlock block, IDLField field, List<string> sizeParts, List<string> returnVecs, string underscorePath, string accessorPath, string prefix = "")
        {
            var fieldPathUnderscore = underscorePath == "" ? field.Name : underscorePath + "_" + field.Name;
            var fieldPathAccessor = accessorPath == "" ? field.Name : accessorPath + "." + field.Name;

            if (field.IsArray)
            {
                var typeName = field.GetInnerRustType("", false);
                if (IsArrayWithFixedSizeItems(field))
                {
                    block.AddLine($"*(pointer as *mut usize) = {fieldPathUnderscore}_count;");
                    block.AddLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                    block.AddLine($"let {fieldPathUnderscore} = Vec::<{typeName}>::from_raw_parts(pointer as *mut {typeName}, {fieldPathUnderscore}_count, {fieldPathUnderscore}_count);");
                    block.AddLine($"let pointer = pointer.offset({fieldPathUnderscore}_count as isize * mem::size_of::<{typeName}>() as isize);");
                    sizeParts.Add($"mem::size_of::<usize>() + {fieldPathUnderscore}_count * mem::size_of::<{typeName}>()");
                    returnVecs.Add($"ManuallyDrop::new({fieldPathUnderscore})");
                }
                else
                {
                    block.AddLine($"*(pointer as *mut usize) = {prefix}{fieldPathUnderscore}.len();");
                    block.AddLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                    block.AddLine($"let mut _{fieldPathUnderscore}_size: usize = mem::size_of::<usize>();");
                    var forBlock = block.AddBlock($"for item in {prefix}{fieldPathUnderscore}.iter()");
                    if (field.Type == IDLField.FieldType.String)
                    {
                        forBlock.AddLine("let item_size = item.len();");
                        forBlock.AddLine("*(pointer as *mut usize) = item_size;");
                        forBlock.AddLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                        forBlock.AddLine("core::ptr::copy(item.as_ptr(), pointer, item_size);");
                        forBlock.AddLine("let item_size = mem::size_of::<usize>() + item_size;");
                    }
                    else if (field.Type == IDLField.FieldType.CustomType || field.Type == IDLField.FieldType.OneOfType)
                    {
                        forBlock.AddLine("let item_size = item.create_at_address(pointer);");
                    }
                    else throw new ArgumentException("Array with variable sized items must be either of type string, custom type or oneOf");
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
                        block.AddLine($"let _{fieldPathUnderscore}_length = {fieldPathUnderscore}.len();");
                        block.AddLine($"*(pointer as *mut usize) = _{fieldPathUnderscore}_length;");
                        block.AddLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                        block.AddLine($"core::ptr::copy({fieldPathUnderscore}.as_ptr(), pointer, _{fieldPathUnderscore}_length);");
                        block.AddLine($"let pointer = pointer.offset(_{fieldPathUnderscore}_length as isize);");

                        sizeParts.Add($"mem::size_of::<usize>() + _{fieldPathUnderscore}_length");
                        break;

                    case IDLField.FieldType.CustomType:
                        foreach (var typeField in field.CustomType.Fields.Values)
                        {
                            AppendTypeWrite(block, typeField, sizeParts, returnVecs,
                                underscorePath == "" ? field.Name : underscorePath + "_" + field.Name,
                                accessorPath == "" ? field.Name : accessorPath + "." + field.Name);
                        }
                        break;
                }
            }
        }

        public static void AppendTypeRead(SourceGenerator.SourceBlock block, string implName, IDLField field, List<string> sizeParts, string underscorePath, string accessorPath, string prefix = "")
        {
            var fieldPathUnderscore = underscorePath == "" ? field.Name : underscorePath + "_" + field.Name;
            var fieldPathAccessor = accessorPath == "" ? field.Name : accessorPath + "." + field.Name;

            if (field.IsArray)
            {
                var typeName = field.GetInnerRustType("", false);
                block.AddLine($"let {fieldPathUnderscore}_count = *(pointer as *mut usize);");
                block.AddLine($"let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                if (IsArrayWithFixedSizeItems(field))
                {
                    block.AddLine($"let {fieldPathUnderscore} = Vec::<{typeName}>::from_raw_parts(pointer as *mut {typeName}, {fieldPathUnderscore}_count, {fieldPathUnderscore}_count);");
                    block.AddLine($"let pointer = pointer.offset({fieldPathUnderscore}_count as isize * mem::size_of::<{typeName}>() as isize);");
                    block.AddLine($"(*object).{fieldPathAccessor} = {fieldPathUnderscore};");
                    sizeParts.Add($"mem::size_of::<usize>() + {fieldPathUnderscore}_count * mem::size_of::<{typeName}>()");
                }
                else
                {
                    block.AddLine($"let mut _{fieldPathUnderscore}_size: usize = mem::size_of::<usize>();");
                    //::
                    block.AddLine($"let mut _{fieldPathUnderscore}_vec: Vec<{(field.Type == IDLField.FieldType.OneOfType ? implName : "")}{typeName}> = Vec::with_capacity(_{fieldPathUnderscore}_size);");
                    var forBlock = block.AddBlock($"for _ in 0..{fieldPathUnderscore}_count");
                    if (field.Type == IDLField.FieldType.String)
                    {
                        forBlock.AddLine("let item_size = *(pointer as *mut usize);");
                        forBlock.AddLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                        forBlock.AddLine("let item = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, item_size)).to_owned();");
                        forBlock.AddLine($"_{fieldPathUnderscore}_vec.push(item);");
                        forBlock.AddLine("let item_size = mem::size_of::<usize>() + item_size;");
                    }
                    else if (field.Type == IDLField.FieldType.CustomType)
                    {
                        forBlock.AddLine($"let (item_size, item) = {typeName}::get_from_address(pointer);");
                        forBlock.AddLine($"_{fieldPathUnderscore}_vec.push(item);");
                    }
                    else if (field.Type == IDLField.FieldType.OneOfType)
                    {
                        forBlock.AddLine($"let (item_size, item) = {implName}{typeName}::get_from_address(pointer);");
                        forBlock.AddLine($"_{fieldPathUnderscore}_vec.push(item);");
                    }
                    else throw new ArgumentException("Array with variable sized items must be either of type string, custom type or oneOf");
                    forBlock.AddLine("let pointer = pointer.offset(item_size as isize);");
                    forBlock.AddLine($"_{fieldPathUnderscore}_size += item_size;");
                    block.AddLine($"(*object).{fieldPathAccessor} = _{fieldPathUnderscore}_vec;");
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
                        //block.AddLine($"(*object).{fieldPathAccessor} = {fieldPathUnderscore};");
                        break;

                    case IDLField.FieldType.String:
                        block.AddLine($"let _{fieldPathUnderscore}_length = *(pointer as *mut usize);");
                        block.AddLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                        block.AddLine($"(*object).{fieldPathAccessor} = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, _{fieldPathUnderscore}_length)).to_owned();");
                        block.AddLine($"let pointer = pointer.offset(_{fieldPathUnderscore}_length as isize);");

                        sizeParts.Add($"mem::size_of::<usize>() + _{fieldPathUnderscore}_length");
                        break;

                    case IDLField.FieldType.CustomType:
                        foreach (var typeField in field.CustomType.Fields.Values)
                        {
                            AppendTypeRead(block, implName, typeField, sizeParts,
                                underscorePath == "" ? field.Name : underscorePath + "_" + field.Name,
                                accessorPath == "" ? field.Name : accessorPath + "." + field.Name);
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
                AppendTypeWrite(body, field, sizeParts, returnVecs, "", "");
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

        private static void GenerateRead(SourceGenerator.SourceBlock block, string implName, List<IDLField> fields)
        {
            var body = block.AddBlock($"pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, &'static mut Self)");
            body.AddLine($"let object: *mut {implName} = mem::transmute(pointer);");
            body.AddLine($"let pointer = pointer.offset(mem::size_of::<{implName}>() as isize);");

            var sizeParts = new List<string>();
            sizeParts.Add($"mem::size_of::<{implName}>()");

            foreach (var field in fields)
            {
                body.AddBlank();
                body.AddLine($"// {field.Name}");
                AppendTypeRead(body, implName, field, sizeParts, "", "");
            }

            // return
            var sizeString = string.Join(" + ", sizeParts);
            body.AddBlank();
            body.AddLine("// return");
            body.AddLine($"({sizeString}, object.as_mut().unwrap())");
        }

        internal static void GenerateCall(SourceGenerator source, IDLCall call, int message_id)
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
                TypeGenerator.GenerateType(source, type);

                var block = source.AddBlock($"impl {structName}");
                GenerateWrite(block, structName, call.Parameters.Values.ToList());
                GenerateRead(block, structName, call.Parameters.Values.ToList());
            }

            if (call.ReturnValues.Count > 0)
            {
                var structName = $"{callName.ToPascal()}Returns";

                var type = new IDLType
                {
                    Name = structName,
                    Fields = call.ReturnValues
                };
                TypeGenerator.GenerateType(source, type);

                var block = source.AddBlock($"impl {structName}");
                GenerateWrite(block, structName, call.ReturnValues.Values.ToList());
                GenerateRead(block, structName, call.ReturnValues.Values.ToList());
            }
        }
    }
}
