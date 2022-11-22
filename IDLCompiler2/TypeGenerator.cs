using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace IDLCompiler
{
    internal class TypeGenerator
    {
        public static void GenerateType(FileStream output, IDLType type)
        {
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
                var line = block.AddLine($"{field.Name}: {field.GetRustType()}");
                line.CommaAfter = true;
            }

            using (var writer = new StreamWriter(output)) 
            {
                writer.Write(source.GetSource());
            }
        }
    }
}
