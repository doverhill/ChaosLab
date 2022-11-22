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
                var line = block.AddLine($"{field.Name}: {field.GetRustType(type.Name)}");
                line.CommaAfter = true;
            }

            using (var writer = new StreamWriter(output, leaveOpen: true)) 
            {
                writer.WriteLine(source.GetSource());
                writer.WriteLine();
            }
        }

        private static string GetOneOfLine(IDLField.OneOfOption option)
        {
            if (option.Type == IDLField.FieldType.None) return "TypeNone";
            if (option.Type == IDLField.FieldType.CustomType) return $"Type{option.CustomType.Name}({option.CustomType.Name})";
            if (option.Type == IDLField.FieldType.OneOfType) throw new ArgumentException("OneOf type is not allowed as a one-of option");

            return $"Type{option.Type}({IDLField.GetRustType(option.Type, null, null, null, false, null)})";
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

            using (var writer = new StreamWriter(output, leaveOpen: true))
            {
                writer.WriteLine(source.GetSource());
                writer.WriteLine();
            }
        }
    }
}
