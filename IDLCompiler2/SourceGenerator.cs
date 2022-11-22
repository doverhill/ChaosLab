using System.Collections.Generic;
using System.Linq;

namespace IDLCompiler
{
    internal class SourceGenerator
    {
        public class SourceBlock
        {
            private List<SourceBlock> _blocks;
            private string _line;

            public bool CommaAfter;
            public bool SemiColorAfter;

            public void AddBlank()
            {
                _blocks.Add(Blank());
            }

            public static SourceBlock Blank()
            {
                return new SourceBlock { _blocks = null, _line = null };
            }

            public SourceBlock AddLine(string value)
            {
                var block = Line(value);
                _blocks.Add(block);
                return block;
            }

            public static SourceBlock Line(string value)
            {
                return new SourceBlock { _blocks = null, _line = value };
            }

            public SourceBlock AddBlock(string value)
            {
                var block = Block(value);
                _blocks.Add(block);
                return block;
            }

            public static SourceBlock Block(string value)
            {
                return new SourceBlock { _blocks = new(), _line = value };
            }

            public string GetSource(int indent)
            {
                if (_blocks == null && _line == null) return "\n";

                var result = new string(' ', 4 * indent) + _line;
                if (_blocks != null)
                {
                    result += " {\n";
                    foreach ( var block in _blocks)
                    {
                        result += block.GetSource(indent + 1);
                        if (block.CommaAfter) result += ",";
                        if (block.SemiColorAfter) result += ";";
                        result += "\n";
                    }
                    result += new string(' ', 4 * indent) + "}";
                    if (CommaAfter) result += ",";
                    if (SemiColorAfter) result += ";";
                }
                return result;
            }
        }

        public List<SourceBlock> Blocks = new();

        public void AddBlank()
        {
            var block = SourceBlock.Blank();
            Blocks.Add(block);
        }

        public void AddLine(string value)
        {
            var block = SourceBlock.Line(value);
            Blocks.Add(block);
        }

        public SourceBlock AddBlock(string value)
        {
            var block = SourceBlock.Block(value);
            Blocks.Add(block);
            return block;
        }

        public string GetSource()
        {
            return string.Join("\n\n", Blocks.Select(b => b.GetSource(0)));
        }
    }
}
