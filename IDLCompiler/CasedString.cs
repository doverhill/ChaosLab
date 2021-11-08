using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal class CasedString
    {
        private List<string> parts;

        public CasedString(List<string> parts)
        {
            this.parts = parts;
        }

        public static CasedString FromPascal(string pascalString)
        {
            var wordIndices = new List<int>();
            var index = 0;
            foreach (var c in pascalString)
            {
                if (char.IsUpper(c)) wordIndices.Add(index);
                index++;
            }
            var result = "";
            var parts = new List<string>();
            for (index = 0; index < wordIndices.Count; index++)
            {
                if (index == wordIndices.Count - 1)
                {
                    // last word
                    var word = pascalString.Substring(wordIndices[index]);
                    parts.Add(word.ToLower());
                }
                else
                {
                    var word = pascalString.Substring(wordIndices[index], wordIndices[index + 1] - wordIndices[index]);
                    parts.Add(word.ToLower());
                }
            }
            return new CasedString(parts);
        }

        public static CasedString FromSnake(string snakeString)
        {
            var parts = snakeString.Split("_", StringSplitOptions.RemoveEmptyEntries).Select(p => p.ToLower()).ToList();
            return new CasedString(parts);
        }

        private string CapitalizeFirst(string part)
        {
            if (part.Length == 1) return part.ToUpper();
            return part.Substring(0, 1).ToUpper() + part.Substring(1);
        }

        public string ToPascal()
        {
            return string.Join(null, parts.Select(p => CapitalizeFirst(p)));
        }

        public string ToSnake()
        {
            return string.Join("_", parts);
        }

        public string ToScreamingSnake()
        {
            return string.Join("_", parts.Select(p => p.ToUpper()));
        }
    }
}
