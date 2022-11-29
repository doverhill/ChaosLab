using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal static class TypeGenerator
    {
        public static void GenerateType(SourceGenerator source, IDLType type)
        {
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

            // strings
            foreach (var field in type.Fields.Values.Where(t => !t.IsArray && t.Type == IDLField.FieldType.String))
            {
                functionBlock.AddBlank();
                functionBlock.AddLine($"// string {field.Name}");
                functionBlock.AddLine($"let mut len = self.{field.Name}.len();");
                functionBlock.AddLine("*(pointer as *mut usize) = len;");
                functionBlock.AddLine($"core::ptr::copy(self.{field.Name}.as_ptr(), pointer, len);");
                functionBlock.AddLine("len = ((len + 7) / 8) * 8;");
                functionBlock.AddLine("pointer = pointer.offset(len as isize);");
                functionBlock.AddLine("size += mem::size_of::<usize>() + len;");
            }

            // custom type
            foreach (var field in type.Fields.Values.Where(t => !t.IsArray && t.Type == IDLField.FieldType.CustomType))
            {
                functionBlock.AddBlank();
                functionBlock.AddLine($"// type {field.CustomType.Name} {field.Name}");
                functionBlock.AddLine("// TODO");
            }

            // one of
            foreach (var field in type.Fields.Values.Where(t => !t.IsArray && t.Type == IDLField.FieldType.OneOfType))
            {
                functionBlock.AddBlank();
                functionBlock.AddLine($"// one of {field.Name}");
                functionBlock.AddLine("// TODO");
            }

            // arrays
            foreach (var field in type.Fields.Values.Where(t => t.IsArray))
            {
                var innerType = IDLField.GetRustType(field.Type, field.CustomType, type.Name, field.Name, field.CustomEnumList, false);

                functionBlock.AddBlank();
                functionBlock.AddLine($"// array {field.Name}");
                functionBlock.AddLine($"let len = self.{field.Name}.len();");
                functionBlock.AddLine("*(pointer as *mut usize) = len;");
                functionBlock.AddLine("pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                functionBlock.AddLine($"core::ptr::copy(self.{field.Name}.as_ptr(), pointer as *mut {innerType}, len);");
                functionBlock.AddLine($"pointer = pointer.offset(len as isize * mem::size_of::<{innerType}>() as isize);");
                functionBlock.AddLine($"size += mem::size_of::<usize>() + len * mem::size_of::<{innerType}>();");

                var forBlock = functionBlock.AddBlock($"for item in self.{field.Name}.iter()");
                forBlock.AddLine("let item_size = item.write_references_at(pointer);");
                forBlock.AddLine("pointer = pointer.offset(item_size as isize);");
                forBlock.AddLine("size += item_size;");
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

            // strings
            foreach (var field in type.Fields.Values.Where(t => !t.IsArray && t.Type == IDLField.FieldType.String))
            {
                functionBlock.AddBlank();
                functionBlock.AddLine($"// string {field.Name}");
                functionBlock.AddLine("let mut len = *(pointer as *const usize);");
                functionBlock.AddLine("pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                functionBlock.AddLine("let mut assign = ManuallyDrop::new(String::from_raw_parts(pointer, len, len);");
                functionBlock.AddLine("core::ptr::write(addr_of_mut!((*object_pointer).name), ManuallyDrop::take(&mut assign));");
                functionBlock.AddLine("len = ((len + 7) / 8) * 8;");
                functionBlock.AddLine("pointer = pointer.offset(len as isize);");
                functionBlock.AddLine("size += mem::size_of::<usize>() + len;");
            }

            // custom type
            foreach (var field in type.Fields.Values.Where(t => !t.IsArray && t.Type == IDLField.FieldType.CustomType))
            {
                functionBlock.AddBlank();
                functionBlock.AddLine($"// type {field.CustomType.Name} {field.Name}");
                functionBlock.AddLine("// TODO");
            }

            // one of
            foreach (var field in type.Fields.Values.Where(t => !t.IsArray && t.Type == IDLField.FieldType.OneOfType))
            {
                functionBlock.AddBlank();
                functionBlock.AddLine($"// one of {field.Name}");
                functionBlock.AddLine("// TODO");
            }

            // arrays
            foreach (var field in type.Fields.Values.Where(t => t.IsArray))
            {
                var innerType = IDLField.GetRustType(field.Type, field.CustomType, type.Name, field.Name, field.CustomEnumList, false);

                functionBlock.AddBlank();
                functionBlock.AddLine($"// array {field.Name}");
                functionBlock.AddLine($"let len = *(pointer as *const usize);");
                functionBlock.AddLine("pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                functionBlock.AddLine($"let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut {innerType}, len, len);");
                functionBlock.AddLine($"core::ptr::writer(addr_of_mut!((*object_pointer).{field.Name}), ManuallyDrop::take(&mut assign));");
                functionBlock.AddLine($"size += mem::size_of::<usize>() + len * mem::size_of::<{innerType}>();");
                functionBlock.AddLine($"let mut references_pointer = pointer.offset(len as isize * mem::size_of::<{innerType}>() as isize);");

                var forBlock = functionBlock.AddBlock($"for item in (*object_pointer).{field.Name}.iter()");
                forBlock.AddLine($"let item_size = {innerType}::reconstruct_at(pointer as *mut {innerType}, references_pointer);");
                forBlock.AddLine($"pointer = pointer.offset(mem::size_of::<{innerType}>() as isize);");
                forBlock.AddLine("references_pointer = references_pointer.offset(item_size as isize);");
                forBlock.AddLine("size += item_size;");

                functionBlock.AddLine("pointer = references_pointer;");
            }

            //if (option.Type == IDLField.FieldType.CustomType)
            //{
            //    caseBlock.AddLine("value.write_at(pointer)");
            //}
            //else if (option.Type == IDLField.FieldType.String)
            //{
            //    caseBlock.AddLine("let mut len = value.len();");
            //    caseBlock.AddLine("*(pointer as *mut usize) = len;");
            //    caseBlock.AddLine("pointer = pointer.offset(mem::size_of::<usize>() as isize);");
            //    caseBlock.AddLine("core::ptr::copy(value.as_ptr(), pointer, len);");
            //    caseBlock.AddLine("len = ((len + 7) / 8) * 8;");
            //    caseBlock.AddLine("mem::size_of::<usize>() + len");
            //}
            //else if (option.Type == IDLField.FieldType.OneOfType)
            //{
            //    caseBlock.AddLine("// FIXME: check this");
            //    caseBlock.AddLine("value.write_at(pointer)");
            //}
            //else
            //{
            //    caseBlock.AddLine("0");
            //}

            functionBlock.AddBlank();
            functionBlock.AddLine("size");
        }
    }
}
