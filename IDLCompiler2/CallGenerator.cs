using System;
using System.Collections.Generic;
using System.Diagnostics.Tracing;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal class CallGenerator
    {
        private static void GenerateWrite(SourceGenerator.SourceBlock block, List<IDLField> field)
        {

        }

        private static void GenerateRead(SourceGenerator.SourceBlock block, List<IDLField> fields)
        {

        }

        internal static void GenerateCall(FileStream output, IDLCall call, int message_id)
        {
            var callName = CasedString.FromSnake(call.Name);

            if (call.Parameters.Count > 0)
            {
                var type = new IDLType
                {
                    Name = $"{callName.ToPascal()}Parameters",
                    Fields = call.Parameters
                };
                TypeGenerator.GenerateType(output, type);

                var source = new SourceGenerator();
                var block = source.AddBlock($"impl {callName.ToPascal()}Parameters");
                GenerateWrite(block, call.Parameters.Values.ToList());
                GenerateRead(block, call.Parameters.Values.ToList());

                using (var writer = new StreamWriter(output, leaveOpen: true))
                {
                    writer.WriteLine(source.GetSource());
                    writer.WriteLine();
                }
            }

            if (call.ReturnValues.Count > 0)
            {
                var type = new IDLType
                {
                    Name = $"{callName.ToPascal()}Returns",
                    Fields = call.ReturnValues
                };
                TypeGenerator.GenerateType(output, type);

                var source = new SourceGenerator();
                var block = source.AddBlock($"impl {callName.ToPascal()}Returns");
                GenerateWrite(block, call.ReturnValues.Values.ToList());
                GenerateRead(block, call.ReturnValues.Values.ToList());

                using (var writer = new StreamWriter(output, leaveOpen: true))
                {
                    writer.WriteLine(source.GetSource());
                    writer.WriteLine();
                }
            }
        }
    }
}
