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

        public void EnterBlock()
        {
            indent++;
        }

        public void LeaveBlock()
        {
            indent--;
        }

        public void WriteLine(string line)
        {
            Console.WriteLine("writing '" + line + "' at level " + indent);
            writer.Write(new string(' ', IndentationSteps * indent));
            writer.WriteLine(line);
        }

        public void BlankLine()
        {
            writer.WriteLine();
        }
    }
}
