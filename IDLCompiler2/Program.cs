using System;
using System.IO;
using System.Text.Json;

namespace IDLCompiler
{
    public class IDLCompiler
    {
        public static void Main(string[] args)
        {
            if (args.Length < 1)
            {
                Console.WriteLine("Error: Use with <IDL file>");
                return;
            }

            var filename = args[0];

            try
            {
                ProcessIDLFile(filename);
                Console.WriteLine("IDL compiler: done");
            }
            catch (Exception e)
            {
                Console.WriteLine("IDL compiler: ERROR: " + e.Message);
            }
        }

        private static void ProcessIDLFile(string filename)
        {
            var fileContents = File.ReadAllText(filename);

            JsonSerializerOptions options = new()
            {
                AllowTrailingCommas = true,
                IncludeFields = true,
                WriteIndented = false,
                PropertyNamingPolicy = null,
                DictionaryKeyPolicy = null,
                ReadCommentHandling = JsonCommentHandling.Skip,
                NumberHandling = System.Text.Json.Serialization.JsonNumberHandling.Strict,
                Converters = { new System.Text.Json.Serialization.JsonStringEnumConverter() }
            };

            IDL idl = null;
            try
            {
                idl = JsonSerializer.Deserialize<IDL>(fileContents, options);
            }
            catch (Exception e)
            {
                throw new ArgumentException("Failed to read IDL file: " + e.Message);
            }
            if (idl == null) throw new ArgumentException("Failed to read IDL file. File empty?");

            idl.Validate();
            idl.Dump();

            if (idl.EnumLists.Count > 0)
            {
                Console.WriteLine("Generating enums");
                Directory.CreateDirectory("enums");
                foreach (var enumList in idl.EnumLists)
                {
                    var enumName = CasedString.FromPascal(enumList.Key);
                    using (var output = new FileStream($"enums/{enumName.ToSnake()}.rs", FileMode.Create))
                    {
                        EnumGenerator.GenerateEnum(output, enumList.Value);
                    }
                }
            }

            if (idl.Types.Count > 0)
            {
                Console.WriteLine("Generating types");
                Directory.CreateDirectory("types");
                foreach (var type in idl.Types)
                {
                    var typeName = CasedString.FromPascal(type.Key);
                    using (var output = new FileStream($"types/{typeName.ToSnake()}.rs", FileMode.Create))
                    {
                        TypeGenerator.GenerateType(output, type.Value);
                    }
                }
            }

            var message_id = 1;

            if (idl.FromClient.Count > 0)
            {
                Console.WriteLine("Generating calls from client");
                Directory.CreateDirectory("from_client");
                foreach (var call in idl.FromClient)
                {
                    using (var output = new FileStream($"from_client/{call.Key}.rs", FileMode.Create))
                    {
                        CallGenerator.GenerateCall(output, call.Value, message_id);
                    }
                    message_id++;
                }
            }

            if (idl.FromClient.Count > 0)
            {
                Console.WriteLine("Generating calls from server");
                Directory.CreateDirectory("from_server");
                foreach (var call in idl.FromServer)
                {
                    using (var output = new FileStream($"from_server/{call.Key}.rs", FileMode.Create))
                    {
                        CallGenerator.GenerateCall(output, call.Value, message_id);
                    }
                    message_id++;
                }
            }

            Console.WriteLine("Generating library");
        }
    }
}
