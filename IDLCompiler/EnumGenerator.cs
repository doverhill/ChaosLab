using System;

namespace IDLCompiler
{
    internal static class EnumGenerator
    {
        public static void GenerateEnum(SourceGenerator source, EnumList enumList, bool dataCarrying)
        {
            Console.WriteLine($"    Enum {enumList.Name}");

            if (dataCarrying)
                source.AddLine("#[repr(C, u64)]");
            else
                source.AddLine("#[repr(u64)]");

            var enumBlock = source.AddBlock($"pub enum {enumList.Name}");
            foreach (var option in enumList.Options)
            {
                enumBlock.AddLine($"{option.ToEnumDeclarationString()},");
            }

            source.AddBlank();

            source.AddLine("#[repr(C)]");
            var structBlock = source.AddBlock($"struct {enumList.Name}Struct");
            structBlock.AddLine($"tag: {enumList.Name}StructTag,");
            structBlock.AddLine($"payload: {enumList.Name}StructPayload,");

            source.AddBlank();

            source.AddLine("#[repr(u64)]");
            var tagEnumBlock = source.AddBlock($"enum {enumList.Name}StructTag");
            foreach (var option in enumList.Options)
            {
                tagEnumBlock.AddLine($"{option.ToTagEnumDeclarationString()},");
            }

            source.AddBlank();

            source.AddLine("#[repr(C)]");
            var unionBlock = source.AddBlock($"union {enumList.Name}StructPayload");
            foreach (var option in enumList.Options)
            {
                unionBlock.AddLine(option.ToUnionDeclarationString(enumList.Name));
            }

            source.AddBlank();

            var implBlock = source.AddBlock($"impl {enumList.Name}");
            var writeAtBlock = implBlock.AddBlock("pub unsafe fn write_at(&self, pointer: *mut u8) -> usize");
            writeAtBlock.AddLine("let mut pointer = pointer;");
            writeAtBlock.AddLine($"core::ptr::copy(self, pointer as *mut {enumList.Name}, 1);");
            writeAtBlock.AddLine($"pointer = pointer.offset(mem::size_of::<{enumList.Name}>() as isize);");
            writeAtBlock.AddLine($"mem::size_of::<{enumList.Name}>() + self.write_references_at(pointer)");

            implBlock.AddBlank();

            var writeReferencesAtBlock = implBlock.AddBlock("pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize");
            writeReferencesAtBlock.AddLine("let mut pointer = pointer;");
            var matchBlock = writeReferencesAtBlock.AddBlock("match self");
            foreach (var option in enumList.Options)
            {
                var caseBlock = matchBlock.AddBlock($"{option.ToMatchString(enumList.Name)} =>");
                caseBlock.CommaAfter = true;
                if (option.Type == IDLField.FieldType.CustomType ||
                    option.Type == IDLField.FieldType.OneOfType)
                {
                    caseBlock.AddLine("value.write_references_at(pointer)");
                }
                else if (option.Type == IDLField.FieldType.String)
                {
                    caseBlock.AddLine("let mut len = value.len();");
                    caseBlock.AddLine("*(pointer as *mut usize) = len;");
                    caseBlock.AddLine("pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                    caseBlock.AddLine("core::ptr::copy(value.as_ptr(), pointer, len);");
                    caseBlock.AddLine("len = ((len + 7) / 8) * 8;");
                    caseBlock.AddLine("mem::size_of::<usize>() + len");
                }
                else
                {
                    caseBlock.AddLine("0");
                }
            }

            implBlock.AddBlank();

            var reconstructAtBlock = implBlock.AddBlock($"pub unsafe fn reconstruct_at(object_pointer: *mut {enumList.Name}, references_pointer: *mut u8) -> usize");
            reconstructAtBlock.AddLine($"let object = object_pointer as *mut {enumList.Name}Struct;");
            matchBlock = reconstructAtBlock.AddBlock("match (*object).tag");
            foreach (var option in enumList.Options)
            {
                var caseBlock = matchBlock.AddBlock($"{option.ToEnumStructMatchString(enumList.Name)} =>");
                caseBlock.CommaAfter = true;
                if (option.Type == IDLField.FieldType.CustomType ||
                    option.Type == IDLField.FieldType.OneOfType)
                {
                    caseBlock.AddLine($"{option.CustomType.Name}::reconstruct_at(addr_of_mut!((*object).payload.{EnumList.Option.GetPayloadUnionFieldName(option.Name)}) as *mut {option.CustomType.Name}, references_pointer)");
                }
                else if (option.Type == IDLField.FieldType.String)
                {
                    caseBlock.AddLine("let mut pointer = references_pointer;");
                    caseBlock.AddLine("let mut len = *(pointer as *const usize);");
                    caseBlock.AddLine("pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                    caseBlock.AddLine($"(*object).payload.{EnumList.Option.GetPayloadUnionFieldName(option.Name)} = ManuallyDrop::new(String::from_raw_parts(pointer, len, len));");
                    caseBlock.AddLine("len = ((len + 7) / 8) * 8;");
                    caseBlock.AddLine("mem::size_of::<usize>() + len");
                }
                else
                {
                    caseBlock.AddLine("0");
                }
            }

            source.AddBlank();
        }
    }
}
