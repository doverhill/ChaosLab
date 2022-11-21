using System;
using System.Collections.Generic;
using System.Linq;

namespace IDLCompiler
{
    internal class CasedString
    {
        private List<string> _parts;

        public static bool IsSnake(string s)
        {
            var pascal = FromPascal(s);
            var snake = FromSnake(s);

            return snake.NumberOfParts() >= pascal.NumberOfParts();
        }

        public static bool IsPascal(string s)
        {
            var pascal = FromPascal(s);
            var snake = FromSnake(s);

            return pascal.NumberOfParts() >= snake.NumberOfParts();
        }

        public CasedString(List<string> parts)
        {
            this._parts = parts;
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
            return string.Join(null, _parts.Select(p => CapitalizeFirst(p)));
        }

        public string ToSnake()
        {
            return string.Join("_", _parts);
        }

        public string ToScreamingSnake()
        {
            return string.Join("_", _parts.Select(p => p.ToUpper()));
        }

        public int NumberOfParts() => _parts.Count;
    }
}
