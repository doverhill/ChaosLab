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
            public bool SemiColonAfter;

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
                if (_blocks == null && _line == null) return "\r\n";

                var result = new string(' ', 4 * indent) + _line;
                if (_blocks != null)
                {
                    result += " {\r\n";
                    foreach ( var block in _blocks)
                    {
                        result += block.GetSource(indent + 1);
                    }
                    result += new string(' ', 4 * indent) + "}";
                }
                if (CommaAfter) result += ",";
                if (SemiColonAfter) result += ";";
                result += "\r\n";
                return result;
            }
        }

        public List<SourceBlock> Blocks = new();

        private bool _includeUsings;

        public SourceGenerator(bool includeUsings)
        {
            _includeUsings = includeUsings;
        }

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

        public string GetSource(bool hasTypes, bool hasEnums)
        {
            if (_includeUsings)
            {
                return
                    "#![allow(dead_code)]\r\n" +
                    "#![allow(unused_imports)]\r\n" +
                    "#![allow(unused_variables)]\r\n" +
                    "use core::mem;\r\n" +
                    "use core::mem::ManuallyDrop;\r\n" +
                    "use core::ptr::addr_of_mut;\r\n" +
                    (hasTypes ? "use crate::types::*;\r\n" : "") +
                    (hasEnums ? "use crate::enums::*;\r\n" : "") +
                    "\r\n" +
                    string.Join("", Blocks.Select(b => b.GetSource(0))) + "\r\n";
            }
            else
            {
                return string.Join("", Blocks.Select(b => b.GetSource(0))) + "\r\n";
            }
        }
    }
}
