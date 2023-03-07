using System;
using System.Collections.Generic;
using System.Drawing;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal static class TypeGenerator
    {
        private static bool Copyable(IDLType type) {
            return type.Fields.All(f => {
                if (f.Value.IsArray) return false;
                if (f.Value.Type == IDLField.FieldType.U8 ||
                    f.Value.Type == IDLField.FieldType.U64 ||
                    f.Value.Type == IDLField.FieldType.I64 ||
                    f.Value.Type == IDLField.FieldType.Bool) return true;
                return false;
            });
        }

        public static void GenerateType(SourceGenerator source, IDLType type)
        {
            Console.WriteLine($"    Type {type.Name}");

            // any one of types that we need to implement?
            foreach (var field in type.Fields.Values)
            {
                if (field.Type == IDLField.FieldType.OneOfType)
                {
                    var enumList = new EnumList(IDLField.GetOneOfTypeName(type.Name, field.Name), field.CustomOneOfOptions);
                    EnumGenerator.GenerateEnum(source, enumList, true);
                }
            }

            if (Copyable(type)) source.AddLine("#[derive(Copy, Clone)]");
            var enumBlock = source.AddBlock($"pub struct {type.Name}");
            foreach (var field in type.Fields.Values)
            {
                enumBlock.AddLine($"pub {field.Name}: {IDLField.GetRustType(field.Type, field.CustomType, type.Name, field.Name, field.CustomEnumList, field.IsArray)},");
            }

            source.AddBlank();
            var implBlock = source.AddBlock($"impl {type.Name}");
            var functionBlock = implBlock.AddBlock("pub unsafe fn write_at(&self, pointer: *mut u8) -> usize");
            functionBlock.AddLine("let mut pointer = pointer;");
            functionBlock.AddLine($"core::ptr::copy(self, pointer as *mut {type.Name}, 1);");
            functionBlock.AddLine($"pointer = pointer.offset(mem::size_of::<{type.Name}>() as isize);");
            functionBlock.AddBlank();
            functionBlock.AddLine($"mem::size_of::<{type.Name}>() + self.write_references_at(pointer)");

            implBlock.AddBlank();

            functionBlock = implBlock.AddBlock("pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize");
            functionBlock.AddLine("let mut pointer = pointer;");
            functionBlock.AddLine("let mut size: usize = 0;");

            foreach (var field in type.Fields.Values)
            {
                functionBlock.AddBlank();
                functionBlock.AddLine($"// {field.Type} {field.Name}");

                if (!field.IsArray)
                {
                    if (field.Type == IDLField.FieldType.String)
                    {
                        functionBlock.AddLine($"let mut len = self.{field.Name}.len();");
                        functionBlock.AddLine("*(pointer as *mut usize) = len;");
                        functionBlock.AddLine("pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                        functionBlock.AddLine($"core::ptr::copy(self.{field.Name}.as_ptr(), pointer, len);");
                        functionBlock.AddLine("len = ((len + 7) / 8) * 8;");
                        functionBlock.AddLine("pointer = pointer.offset(len as isize);");
                        functionBlock.AddLine("size += mem::size_of::<usize>() + len;");
                    }
                    else if (field.Type == IDLField.FieldType.CustomType ||
                        field.Type == IDLField.FieldType.OneOfType)
                    {
                        functionBlock.AddLine($"let len = self.{field.Name}.write_references_at(pointer);");
                        functionBlock.AddLine("pointer = pointer.offset(len as isize);");
                        functionBlock.AddLine("size += len;");
                    }
                }
                else
                {
                    var innerType = IDLField.GetRustType(field.Type, field.CustomType, type.Name, field.Name, field.CustomEnumList, false);

                    functionBlock.AddLine($"let len = self.{field.Name}.len();");
                    functionBlock.AddLine("*(pointer as *mut usize) = len;");
                    functionBlock.AddLine("pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                    functionBlock.AddLine($"core::ptr::copy(self.{field.Name}.as_ptr(), pointer as *mut {innerType}, len);");
                    functionBlock.AddLine($"pointer = pointer.offset(len as isize * mem::size_of::<{innerType}>() as isize);");
                    functionBlock.AddLine($"size += mem::size_of::<usize>() + len * mem::size_of::<{innerType}>();");

                    if (field.Type == IDLField.FieldType.String)
                    {
                        var forBlock = functionBlock.AddBlock($"for item in self.{field.Name}.iter()");
                        forBlock.AddLine("let mut len = item.len();");
                        forBlock.AddLine("*(pointer as *mut usize) = len;");
                        forBlock.AddLine("pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                        forBlock.AddLine("core::ptr::copy(item.as_ptr(), pointer, len);");
                        forBlock.AddLine("len = ((len + 7) / 8) * 8;");
                        forBlock.AddLine("pointer = pointer.offset(len as isize);");
                        forBlock.AddLine("size += mem::size_of::<usize>() + len;");

                    }
                    else if (field.Type == IDLField.FieldType.CustomType ||
                        field.Type == IDLField.FieldType.OneOfType)
                    {
                        var forBlock = functionBlock.AddBlock($"for item in self.{field.Name}.iter()");
                        forBlock.AddLine("let item_size = item.write_references_at(pointer);");
                        forBlock.AddLine("pointer = pointer.offset(item_size as isize);");
                        forBlock.AddLine("size += item_size;");
                    }
                }
            }

            functionBlock.AddBlank();
            functionBlock.AddLine("size");

            implBlock.AddBlank();

            functionBlock = implBlock.AddBlock("pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize");
            functionBlock.AddLine($"mem::size_of::<{type.Name}>() + Self::reconstruct_at(object_pointer as *mut {type.Name}, object_pointer.offset(mem::size_of::<{type.Name}>() as isize))");

            implBlock.AddBlank();

            functionBlock = implBlock.AddBlock($"pub unsafe fn reconstruct_at(object_pointer: *mut {type.Name}, references_pointer: *mut u8) -> usize");
            functionBlock.AddLine("let mut pointer = references_pointer;");
            functionBlock.AddLine("let mut size: usize = 0;");

            foreach (var field in type.Fields.Values)
            {
                functionBlock.AddBlank();
                functionBlock.AddLine($"// {field.Type} {field.Name}");

                if (!field.IsArray)
                {
                    if (field.Type == IDLField.FieldType.String)
                    {
                        functionBlock.AddLine("let mut len = *(pointer as *const usize);");
                        functionBlock.AddLine("pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                        functionBlock.AddLine("let mut assign = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));");
                        functionBlock.AddLine($"core::ptr::write(addr_of_mut!((*object_pointer).{field.Name}), ManuallyDrop::take(&mut assign));");
                        functionBlock.AddLine("len = ((len + 7) / 8) * 8;");
                        functionBlock.AddLine("pointer = pointer.offset(len as isize);");
                        functionBlock.AddLine("size += mem::size_of::<usize>() + len;");
                    }
                    else if (field.Type == IDLField.FieldType.CustomType ||
                        field.Type == IDLField.FieldType.OneOfType)
                    {
                        var typeName = field.Type == IDLField.FieldType.CustomType ? field.CustomType.Name : IDLField.GetOneOfTypeName(type.Name, field.Name);
                        functionBlock.AddLine($"let len = {typeName}::reconstruct_at(addr_of_mut!((*object_pointer).{field.Name}), pointer);");
                        functionBlock.AddLine("pointer = pointer.offset(len as isize);");
                        functionBlock.AddLine("size += len;");
                    }
                }
                else
                {
                    var innerType = IDLField.GetRustType(field.Type, field.CustomType, type.Name, field.Name, field.CustomEnumList, false);

                    functionBlock.AddLine($"let len = *(pointer as *const usize);");
                    functionBlock.AddLine("pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                    functionBlock.AddLine($"let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut {innerType}, len, len));");
                    functionBlock.AddLine($"core::ptr::write(addr_of_mut!((*object_pointer).{field.Name}), ManuallyDrop::take(&mut assign));");
                    functionBlock.AddLine($"size += mem::size_of::<usize>() + len * mem::size_of::<{innerType}>();");
                    functionBlock.AddLine($"let mut references_pointer = pointer.offset(len as isize * mem::size_of::<{innerType}>() as isize);");

                    if (field.Type == IDLField.FieldType.String)
                    {
                        var forBlock = functionBlock.AddBlock($"for item in (*object_pointer).{field.Name}.iter()");
                        forBlock.AddLine("let mut len = *(references_pointer as *const usize);");
                        forBlock.AddLine("references_pointer = references_pointer.offset(mem::size_of::<usize>() as isize);");
                        forBlock.AddLine("let mut assign = ManuallyDrop::new(String::from_raw_parts(references_pointer, len, len));");
                        forBlock.AddLine("core::ptr::write(pointer as *mut String, ManuallyDrop::take(&mut assign));");
                        forBlock.AddLine("pointer = pointer.offset(mem::size_of::<String>() as isize);");
                        forBlock.AddLine("len = ((len + 7) / 8) * 8;");
                        forBlock.AddLine("references_pointer = references_pointer.offset(len as isize);");
                        forBlock.AddLine("size += mem::size_of::< usize > () + len;");

                    }
                    else if (field.Type == IDLField.FieldType.CustomType ||
                        field.Type == IDLField.FieldType.OneOfType)
                    {
                        var forBlock = functionBlock.AddBlock($"for item in (*object_pointer).{field.Name}.iter()");
                        forBlock.AddLine($"let item_size = {innerType}::reconstruct_at(pointer as *mut {innerType}, references_pointer);");
                        forBlock.AddLine($"pointer = pointer.offset(mem::size_of::<{innerType}>() as isize);");
                        forBlock.AddLine("references_pointer = references_pointer.offset(item_size as isize);");
                        forBlock.AddLine("size += item_size;");
                    }

                    functionBlock.AddLine("pointer = references_pointer;");
                }
            }

            functionBlock.AddBlank();
            functionBlock.AddLine("size");

            source.AddBlank();
        }
    }
}
