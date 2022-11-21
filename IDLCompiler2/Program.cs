using System;
using System.Collections.Generic;
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

            var message_id = 1;
            foreach (var call in idl.FromClient)
            {
                var callName = CasedString.FromSnake(call.Key);
                using (var output = new FileStream($"from_client/{call.Key}.rs", FileMode.Create))
                {
                    StructGenerator.WriteStruct(output, $"{callName.ToPascal()}Parameters", )
                    CallGenerator.WriteCall(output, message_id);
    
                }
                    message_id++;
            }

        }
    }
}
