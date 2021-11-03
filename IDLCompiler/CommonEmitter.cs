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
        private IDL idl;
        private int indent = 0;
        private const int IndentationSteps = 4;

        public CommonEmitter(IDL idl, StreamWriter writer)
        {
            this.idl = idl;
            this.writer = writer;
        }

        private void WriteIndent()
        {
            writer.Write(new string(' ', IndentationSteps * indent));
        }

        private (string Type, string Name, bool IsList, int? ListCount) ParseField(string field)
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
                case "i32":
                case "i64":
                case "u32":
                case "u64":
                case "f32":
                case "f64":
                case "bool":
                    break;

                case "string":
                    typeName = "[u8; 100]";
                    break;

                case "size":
                    typeName = "u64";
                    break;

                case "datetime":
                    typeName = "u64";
                    break;

                case "byte":
                    typeName = "u8";
                    break;

                default:
                    if (!idl.Types.Any(t => t.Name == typeName)) throw new Exception("Missing type: '" + typeName + "'");
                    break;
            };

            return (typeName, fieldName, isList, listCount);
        }

        public void WriteStruct(string name, List<string> fields, int batchSize)
        {
            WriteIndent(); writer.WriteLine("pub struct " + name + " {"); indent++;

            for (var fieldIndex = 0; fieldIndex < fields.Count; fieldIndex++)
            {
                var field = fields[fieldIndex];
                var lastField = fieldIndex == fields.Count - 1;

                var (fieldType, fieldName, isList, listCount) = ParseField(field);
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
                    WriteIndent(); writer.WriteLine(fieldName + ": " + fieldType + (!lastField ? "," : ""));
                }
            }

            indent--; WriteIndent(); writer.WriteLine("}");
        }

        private string ToSnakeCase(string pascalCase)
        {
            var wordIndices = new List<int>();
            var index = 0;
            foreach (var c in pascalCase)
            {
                if (char.IsUpper(c)) wordIndices.Add(index);
                index++;
            }

            var result = "";
            for (index = 0; index < wordIndices.Count; index++)
            {
                if (index == wordIndices.Count - 1)
                {
                    // last word
                    var word = pascalCase.Substring(wordIndices[index]);
                    result += word.ToLower();
                }
                else
                {
                    var word = pascalCase.Substring(wordIndices[index], wordIndices[index + 1] - wordIndices[index]);
                    result += word.ToLower() + "_";
                }
            }

            return result;
        }

        private string GetParameterString(string parameter)
        {
            var (fieldType, fieldName, isList, listCount) = ParseField(parameter);
            return fieldName + ": " + fieldType;
        }

        private string GetParametersString(IDLCall call)
        {
            if (call.Parameters != null && call.Parameters.Count > 0)
            {
                return string.Join(", ", call.Parameters.Select(p => GetParameterString(p)));
            }

            return "";
        }

        private string GetReturnString(string ret)
        {
            var (fieldType, fieldName, isList, listCount) = ParseField(ret);
            return fieldType;
        }

        private string GetReturnsString(IDLCall call)
        {
            if (call.Returns != null && call.Returns.Count > 0)
            {
                if (call.Returns.Count == 1)
                {
                    return " -> " + GetReturnString(call.Returns[0]);
                }
                else
                {
                    return " -> (" + string.Join(", ", call.Returns.Select(r => GetReturnString(r))) + ")";
                }
            }
            return "";
        }

        public void WriteCall(IDLCall call)
        {
            // generate safe call, copies struct
            WriteIndent(); writer.WriteLine("pub fn " + ToSnakeCase(call.Name) + "(" + GetParametersString(call) + ")" + GetReturnsString(call) + " {"); indent++;

            indent--; WriteIndent(); writer.WriteLine("}");

            writer.WriteLine();

            // generate unsafe faster call, returns pointer to struct
            WriteIndent(); writer.WriteLine("unsafe pub fn " + ToSnakeCase(call.Name) + "_raw(" + GetParametersString(call) + ")" + GetReturnsString(call) + " {"); indent++;

            indent--; WriteIndent(); writer.WriteLine("}");
        }
    }
}
