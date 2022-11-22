using System.IO;

namespace IDLCompiler
{
    internal class EnumGenerator
    {
        public static void GenerateEnum(FileStream output, EnumList enumList)
        {
            var source = new SourceGenerator();
            var block = source.AddBlock($"enum {enumList.Name}");
            foreach (var item in enumList.Options )
            {
                var line = block.AddLine(item);
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
