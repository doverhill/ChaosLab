namespace IDLCompiler
{
    internal class StructuredWriter
    {
        private const int IndentationSteps = 4;

        private int indent = 0;
        private StreamWriter writer;

        public StructuredWriter(StreamWriter writer)
        {
            this.writer = writer;
        }

        //public void EnterBlock()
        //{
        //    indent++;
        //}

        public void CloseScope(string append = null)
        {
            indent--;
            WriteLine("}" + (append ?? ""));
        }

        public void WriteLine(string line, bool openScope = false)
        {
            writer.Write(new string(' ', IndentationSteps * indent));
            writer.WriteLine(line + (openScope ? " {" : ""));
            if (openScope) indent++;
        }

        public void BlankLine()
        {
            writer.WriteLine();
        }
    }
}
