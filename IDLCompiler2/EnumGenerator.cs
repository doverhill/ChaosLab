using System.IO;

namespace IDLCompiler
{
    internal class EnumGenerator
    {
        public static void GenerateEnum(SourceGenerator source, EnumList enumList)
        {
            var block = source.AddBlock($"pub enum {enumList.Name}");
            foreach (var item in enumList.Options )
            {
                var line = block.AddLine(item);
                line.CommaAfter = true;
            }
        }
    }
}
