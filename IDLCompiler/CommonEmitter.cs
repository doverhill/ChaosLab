using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal class CommonEmitter
    {
        private StreamWriter writer;
        private int indent = 0;
        private const int IndentationSteps = 4;

        public CommonEmitter(StreamWriter writer)
        {
            this.writer = writer;
        }

        private void WriteIndent()
        {
            writer.Write(new string(' ', IndentationSteps * indent));
        }

        public void FileIntro(string nameSpace)
        {
            WriteIndent(); writer.WriteLine("using Core;");
            WriteIndent(); writer.WriteLine("using System.Collections.Generic;");
            writer.WriteLine();
            WriteIndent(); writer.WriteLine("namespace " + nameSpace);
            WriteIndent(); writer.WriteLine("{"); indent++;
        }

        private (string Type, string Name, bool IsList, int? ListCount) ParseField(IDL idl, string field)
        {
            // supported types:
            // string
            // i32
            // i64
            // f32
            // f64
            // bool
            // typename

            var parts = field.Split(" ");
            if (parts.Length != 2) throw new Exception("Malformed type and name: '" + field + "'");
            var typeName = parts[0];
            var fieldName = parts[1];

            int? listCount = null;
            var isList = false;
            if (typeName.EndsWith("[]"))
            {
                typeName = typeName.Substring(0, typeName.Length - 2);
                isList = true;
            }
            else if (typeName.EndsWith("]"))
            {
                var index = typeName.IndexOf("[");
                listCount = int.Parse(typeName.Substring(index + 1, typeName.Length - index - 1));
                typeName = typeName.Substring(0, index);
                isList = true;
            }

            switch (typeName)
            {
                case "string":
                    typeName = "string";
                    break;

                case "i32":
                    typeName = "int";
                    break;

                case "i64":
                    typeName = "long";
                    break;

                case "f32":
                    typeName = "float";
                    break;

                case "f64":
                    typeName = "double";
                    break;

                case "bool":
                    typeName = "bool";
                    break;

                default:
                    if (!idl.Types.Any(t => t.Name == typeName)) throw new Exception("Missing type: '" + typeName + "'");
                    break;
            };

            return (typeName, fieldName, isList, listCount);
        }

        public void WriteStruct(IDL idl, string name, List<string> fields, int batchSize)
        {
            WriteIndent(); writer.WriteLine("internal struct " + name);
            WriteIndent(); writer.WriteLine("{"); indent++;

            foreach (var field in fields)
            {
                var (fieldType, fieldName, isList, listCount) = ParseField(idl, field);
                if (isList)
                {
                    if (listCount.HasValue)
                    {
                        WriteIndent(); writer.WriteLine(fieldType + "[" + listCount.Value + "] " + fieldName + ";");
                    }
                    else
                    {
                        WriteIndent(); writer.WriteLine(fieldType + "[" + batchSize + "] " + fieldName + ";");
                        WriteIndent(); writer.WriteLine("bool " + fieldName + "Continue;");
                    }
                }
                else
                {
                    WriteIndent(); writer.WriteLine(fieldType + " " + fieldName + ";");
                }
            }

            indent--; WriteIndent(); writer.WriteLine("}");
        }

        public void WriteCall(IDL idl, IDLCall call)
        {
            WriteIndent(); writer.WriteLine("internal class " + call.Name + "Call");
            WriteIndent(); writer.WriteLine("{"); indent++;

            string parameters = "";
            if (call.Parameters != null && call.Parameters.Count > 0)
            {
                parameters = call.Name + "Parameters parameters";
                WriteStruct(idl, call.Name + "Parameters", call.Parameters, call.BatchSize);
                writer.WriteLine();
            }

            string returns = "Optional<Error>";
            if (call.Returns != null && call.Returns.Count > 0)
            {
                // check if any return is a list and in that case that it is the only return
                (string Type, string Name, bool IsList, int? ListCount) returnList = (null, null, false, null);
                foreach (var field in call.Returns)
                {
                    var check = ParseField(idl, field);
                    if (check.IsList) returnList = check;
                }
                if (returnList.Type != null && call.Returns.Count > 1) throw new Exception("If call returns a list, it needs to be the only return");

                if (returnList.Type != null)
                {
                    returns = "ErrorOr<IEnumerable<" + returnList.Type + ">>";
                }
                else
                {
                    returns = "ErrorOr<" + call.Name + "Return>";
                    WriteStruct(idl, call.Name + "Return", call.Returns, call.BatchSize);
                }
                writer.WriteLine();
            }

            // call implementation
            WriteIndent(); writer.WriteLine("public " + returns + " " + call.Name + "(" + parameters + ")");
            WriteIndent(); writer.WriteLine("{"); indent++;

            WriteIndent(); writer.WriteLine("return new " + returns + "(Error.NotImplemented);");

            indent--; WriteIndent(); writer.WriteLine("}");
            //call implementation end

            indent--; WriteIndent(); writer.WriteLine("}");
        }

        public void FileOutro()
        {
            indent--; WriteIndent(); writer.WriteLine("}");
        }
    }
}
