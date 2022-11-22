using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace IDLCompiler
{
    internal class TypeGenerator
    {
        public static void GenerateType(FileStream output, IDLType type)
        {
            foreach (var field in type.Fields.Values)
            {
                if (field.Type == IDLField.FieldType.OneOfType)
                {
                    TypeGenerator.GenerateOneOfType(output, $"{type.Name}{CasedString.FromSnake(field.Name).ToPascal()}Enum", field.CustomOneOfOptions);
                }
            }

            var source = new SourceGenerator();
            var block = source.AddBlock($"struct {type.Name}");

            var fields = type.Fields.Values.ToList();
            var inheritFrom = type.GetInheritsFrom();
            if (inheritFrom != null)
            {
                fields = inheritFrom.Fields.Values.ToList().Concat(fields).ToList();
            }

            foreach (var field in fields)
            {
                var line = block.AddLine($"{field.Name}: {field.GetRustType(type.Name, false)}");
                line.CommaAfter = true;
            }

            using (var writer = new StreamWriter(output, leaveOpen: true))
            {
                writer.WriteLine(source.GetSource());
            }
        }

        private static string GetOneOfLine(IDLField.OneOfOption option)
        {
            if (option.Type == IDLField.FieldType.None) return "TypeNone";
            if (option.Type == IDLField.FieldType.CustomType) return $"Type{option.CustomType.Name}({option.CustomType.Name})";
            if (option.Type == IDLField.FieldType.OneOfType) throw new ArgumentException("OneOf type is not allowed as a one-of option");

            return $"Type{option.Type}({IDLField.GetRustType(option.Type, null, null, null, false, null, false)})";
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

        private static void AddCase(SourceGenerator.SourceBlock block, string enumName, IDLField.OneOfOption option)
        {
            switch (option.Type)
            {
                case IDLField.FieldType.CustomType:
                    break;

                case IDLField.FieldType.OneOfType:
                    break;

                case IDLField.FieldType.Enum:
                    throw new ArgumentException("Enum of enums not allowed");
            }

            block.AddLine("size");
        }

        public static void GenerateOneOfType(FileStream output, string enumName, List<IDLField.OneOfOption> oneOfOptions)
        {
            var source = new SourceGenerator();
            var block = source.AddBlock($"enum {enumName}");

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

            var body = block.AddBlock("pub unsafe fn create_at_address(&self, pointer: *mut u8) -> usize");
            body.AddLine($"let mut size: usize = mem::size_of::<{enumName}>();");
            body.AddLine($"core::ptr::copy(self as *const {enumName}, pointer as *mut {enumName}, 1);");
            body.AddBlank();

            var match = body.AddBlock("match self");
            foreach (var option in oneOfOptions)
            {
                var caseBlock = match.AddBlock($"{enumName}::{GetOneOfOption(option)} =>");
                caseBlock.CommaAfter = true;
                AddCase(caseBlock, enumName, option);
            }

            using (var writer = new StreamWriter(output, leaveOpen: true))
            {
                writer.WriteLine(source.GetSource());
            }
        }
    }
}
