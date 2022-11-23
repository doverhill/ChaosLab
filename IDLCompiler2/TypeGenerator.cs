using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace IDLCompiler
{
    internal class TypeGenerator
    {
        public static void GenerateType(SourceGenerator source, IDLType type)
        {
            foreach (var field in type.Fields.Values)
            {
                if (field.Type == IDLField.FieldType.OneOfType)
                {
                    TypeGenerator.GenerateOneOfType(source, $"{type.Name}{CasedString.FromSnake(field.Name).ToPascal()}Enum", field.CustomOneOfOptions);
                }
            }

            var block = source.AddBlock($"pub struct {type.Name}");

            var fields = type.Fields.Values.ToList();
            var inheritFrom = type.GetInheritsFrom();
            if (inheritFrom != null)
            {
                fields = inheritFrom.Fields.Values.ToList().Concat(fields).ToList();
            }

            foreach (var field in fields)
            {
                var allowMutPointer = !CallGenerator.IsFixedSize(field, false);
                var line = block.AddLine($"pub {field.Name}: {field.GetRustType(type.Name, false, allowMutPointer)}");
                line.CommaAfter = true;
            }

            var impl = source.AddBlock($"impl {type.Name}");
            var body = impl.AddBlock("pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize");
            body.AddLine("0");

            CallGenerator.GenerateWrite(impl, type.Name, type.Fields.Values.ToList());
            CallGenerator.GenerateRead(impl, type.Name, type.Fields.Values.ToList());
        }

        private static string GetOneOfLine(IDLField.OneOfOption option)
        {
            if (option.Type == IDLField.FieldType.None) return "TypeNone";
            if (option.Type == IDLField.FieldType.CustomType) return $"Type{option.CustomType.Name}(*mut {option.CustomType.Name})";
            if (option.Type == IDLField.FieldType.OneOfType) throw new ArgumentException("OneOf type is not allowed as a one-of option");

            return $"Type{option.Type}({IDLField.GetRustType(option.Type, null, null, null, false, null, false, false)})";
        }

        private static string GetOneOfOption(IDLField.OneOfOption option)
        {
            if (option.Type == IDLField.FieldType.None) return "TypeNone";
            if (option.Type == IDLField.FieldType.CustomType) return $"Type{option.CustomType.Name}(value)";
            if (option.Type == IDLField.FieldType.OneOfType) throw new ArgumentException("OneOf type is not allowed as a one-of option");

            return $"Type{option.Type}(value)";
        }

        private static string GetOneOfConstantName(IDLField.OneOfOption option)
        {
            if (option.Type == IDLField.FieldType.None) return "OPTION_NONE";
            if (option.Type == IDLField.FieldType.CustomType) return $"OPTION_{option.CustomType.Name.ToUpper()}";
            if (option.Type == IDLField.FieldType.OneOfType) throw new ArgumentException("OneOf type is not allowed as a one-of option");

            return $"OPTION_{option.Type.ToString().ToUpper()}";
        }

        private static void AddReadCase(SourceGenerator.SourceBlock block, string enumName, IDLField.OneOfOption option)
        {
            var sizeParts = new List<string>();
            switch (option.Type)
            {
                case IDLField.FieldType.String:
                    block.AddLine("let value_len = *(pointer as *mut usize);");
                    block.AddLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                    block.AddLine("let value = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pointer as *const u8, value_len)).to_owned();");
                    block.AddLine($"(mem::size_of::<usize>() + value_len, Self::{GetOneOfOption(option)})");
                    break;

                case IDLField.FieldType.CustomType:
                    block.AddLine($"let (size, value) = {option.CustomType.Name}::get_from_address(pointer);");
                    block.AddLine($"(size, Self::{GetOneOfOption(option)})");
                    break;

                case IDLField.FieldType.OneOfType:
                    throw new ArgumentException("OneOf inside OneOf not allowed");

                case IDLField.FieldType.U8:
                    block.AddLine("let value = *(pointer as *mut u8);");
                    block.AddLine($"(mem::size_of::<usize>(), Self::{GetOneOfOption(option)})");
                    break;

                case IDLField.FieldType.U64:
                    block.AddLine("let value = *(pointer as *mut u64);");
                    block.AddLine($"(mem::size_of::<usize>(), Self::{GetOneOfOption(option)})");
                    break;

                case IDLField.FieldType.I64:
                    block.AddLine("let value = *(pointer as *mut i64);");
                    block.AddLine($"(mem::size_of::<usize>(), Self::{GetOneOfOption(option)})");
                    break;

                case IDLField.FieldType.Bool:
                    block.AddLine("let value = *(pointer as *mut usize) == 1;");
                    block.AddLine($"(mem::size_of::<usize>(), Self::{GetOneOfOption(option)})");
                    break;

                case IDLField.FieldType.None:
                    block.AddLine($"(0, Self::{GetOneOfOption(option)})");
                    break;

                default:
                    throw new ArgumentException($"Unsupported type {option.Type} inside OneOf");
            }
        }

        private static void AddWriteCase(SourceGenerator.SourceBlock block, string enumName, IDLField.OneOfOption option)
        {
            block.AddLine($"*(base_pointer as *mut usize) = Self::{GetOneOfConstantName(option)};");

            switch (option.Type)
            {
                case IDLField.FieldType.String:
                    block.AddLine("let value_len = value.len();");
                    block.AddLine("*(pointer as *mut usize) = value_len;");
                    block.AddLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
                    block.AddLine("core::ptr::copy(value.as_ptr(), pointer as *mut u8, value_len);");
                    block.AddLine("mem::size_of::<usize>() + value_len");
                    break;

                case IDLField.FieldType.CustomType:
                    //foreach (var field in option.CustomType.Fields.Values)
                    //{
                    //    CallGenerator.AppendTypeWrite(block, field, sizeParts, null, "", "", "value.");
                    //}
                    block.AddLine($"(value.as_ref().unwrap()).write_at_address(pointer)"); // + string.Join(" + ", sizeParts) + ";");
                    break;

                case IDLField.FieldType.OneOfType:
                    throw new ArgumentException("OneOf inside OneOf not allowed");

                case IDLField.FieldType.U8:
                    block.AddLine("*(pointer as *mut u8) = *value;");
                    block.AddLine("mem::size_of::<usize>()");
                    break;

                case IDLField.FieldType.U64:
                    block.AddLine("*(pointer as *mut u64) = *value;");
                    block.AddLine("mem::size_of::<usize>()");
                    break;

                case IDLField.FieldType.I64:
                    block.AddLine("*(pointer as *mut i64) = *value;");
                    block.AddLine("mem::size_of::<usize>()");
                    break;

                case IDLField.FieldType.Bool:
                    block.AddLine("*(pointer as *mut usize) = if *value { 1 } else { 0 };");
                    block.AddLine("mem::size_of::<usize>()");
                    break;

                case IDLField.FieldType.None:
                    block.AddLine("0");
                    break;

                default:
                    throw new ArgumentException($"Unsupported type {option.Type} inside OneOf");
            }
        }

        public static void GenerateOneOfType(SourceGenerator source, string enumName, List<IDLField.OneOfOption> oneOfOptions)
        {
            var block = source.AddBlock($"pub enum {enumName}");

            foreach (var option in oneOfOptions)
            {
                var line = block.AddLine(GetOneOfLine(option));
                line.CommaAfter = true;
            }

            source.AddBlank();
            block = source.AddBlock($"impl {enumName}");

            var number = 1;
            foreach (var option in oneOfOptions)
            {
                block.AddLine($"pub const {GetOneOfConstantName(option)}: usize = {number++};");
            }
            block.AddBlank();

            var body = block.AddBlock("pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize");
            body.AddLine("let base_pointer = pointer;");
            body.AddLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
            //body.AddLine($"core::ptr::copy(self as *const {enumName}, pointer as *mut {enumName}, 1);");
            //body.AddLine($"let pointer = pointer.offset(mem::size_of::<{enumName}>() as isize);");
            //body.AddLine($"let mut size: usize = mem::size_of::<usize>() + mem::size_of::<{enumName}>();");
            body.AddLine($"let size: usize = mem::size_of::<usize>();");
            body.AddBlank();

            var match = body.AddBlock("let size = match self");
            match.SemiColonAfter = true;
            foreach (var option in oneOfOptions)
            {
                var caseBlock = match.AddBlock($"{enumName}::{GetOneOfOption(option)} =>");
                caseBlock.CommaAfter = true;
                AddWriteCase(caseBlock, enumName, option);
            }

            body.AddBlank();
            body.AddLine("mem::size_of::<usize>() + size");

            body = block.AddBlock("pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, Self)");
            body.AddLine("let enum_type = *(pointer as *mut usize);");
            body.AddLine("let pointer = pointer.offset(mem::size_of::<usize>() as isize);");
            body.AddBlank();

            match = body.AddBlock("let (size, object) = match enum_type");
            match.SemiColonAfter = true;
            foreach (var option in oneOfOptions)
            {
                var caseBlock = match.AddBlock($"Self::{GetOneOfConstantName(option)} =>");
                AddReadCase(caseBlock, enumName, option);
            }
            var panicBlock = match.AddBlock("_ =>");
            panicBlock.AddLine("panic!(\"Unknown enum type\");");

            body.AddBlank();
            body.AddLine("(mem::size_of::<usize>() + size, object)");
        }
    }
}
