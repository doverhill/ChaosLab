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

            Directory.CreateDirectory("src");

            bool hasEnums = false;
            if (idl.EnumLists.Count > 0)
            {
                hasEnums = true;
                Console.WriteLine("Generating enums");
                Directory.CreateDirectory("src/enums");
                using (var modOutput = new FileStream("src/enums/mod.rs", FileMode.Create))
                {
                    var modSource = new SourceGenerator(false);

                    foreach (var enumList in idl.EnumLists)
                    {
                        Console.WriteLine("    " + enumList.Key);
                        var enumName = CasedString.FromPascal(enumList.Key);

                        var codeSource = new SourceGenerator(true);
                        EnumGenerator.GenerateEnum(codeSource, enumList.Value);
                        using (var output = new FileStream($"src/enums/{enumName.ToSnake()}.rs", FileMode.Create))
                        {
                            using (var writer = new StreamWriter(output, leaveOpen: true))
                            {
                                writer.WriteLine(codeSource.GetSource(true));
                            }
                        }

                        modSource.AddLine($"pub mod {enumName.ToSnake()};");
                        modSource.AddLine($"pub use {enumName.ToSnake()}::*;");
                    }

                    using (var writer = new StreamWriter(modOutput, leaveOpen: true))
                    {
                        writer.WriteLine(modSource.GetSource(true));
                    }
                }
            }

            if (idl.Types.Count > 0)
            {
                Console.WriteLine("Generating types");
                Directory.CreateDirectory("src/types");
                using (var modOutput = new FileStream("src/types/mod.rs", FileMode.Create))
                {
                    var modSource = new SourceGenerator(false);

                    foreach (var type in idl.Types)
                    {
                        Console.WriteLine("    " + type.Key);
                        var typeName = CasedString.FromPascal(type.Key);

                        var codeSource = new SourceGenerator(true);
                        TypeGenerator.GenerateType(codeSource, type.Value);
                        using (var output = new FileStream($"src/types/{typeName.ToSnake()}.rs", FileMode.Create))
                        {
                            using (var writer = new StreamWriter(output, leaveOpen: true))
                            {
                                writer.WriteLine(codeSource.GetSource(hasEnums));
                            }
                        }

                        modSource.AddLine($"pub mod {typeName.ToSnake()};");
                        modSource.AddLine($"pub use {typeName.ToSnake()}::*;");
                    }

                    using (var writer = new StreamWriter(modOutput, leaveOpen: true))
                    {
                        writer.WriteLine(modSource.GetSource(hasEnums));
                    }
                }
            }

            //var message_id = 1;

            //if (idl.FromClient.Count > 0)
            //{
            //    Console.WriteLine("Generating calls from client");
            //    Directory.CreateDirectory("src/from_client");
            //    using (var modOutput = new FileStream("src/from_client/mod.rs", FileMode.Create))
            //    {
            //        var modSource = new SourceGenerator(false);

            //        foreach (var call in idl.FromClient)
            //        {
            //            var codeSource = new SourceGenerator(true);
            //            CallGenerator.GenerateCall(codeSource, call.Value, message_id);
            //            using (var output = new FileStream($"src/from_client/{call.Key}.rs", FileMode.Create))
            //            {
            //                using (var writer = new StreamWriter(output, leaveOpen: true))
            //                {
            //                    writer.WriteLine(codeSource.GetSource(hasEnums));
            //                }
            //            }
            //            message_id++;

            //            modSource.AddLine($"pub mod {call.Key};");
            //            modSource.AddLine($"pub use {call.Key}::*;");
            //        }

            //        using (var writer = new StreamWriter(modOutput, leaveOpen: true))
            //        {
            //            writer.WriteLine(modSource.GetSource(hasEnums));
            //        }
            //    }
            //}

            //if (idl.FromClient.Count > 0)
            //{
            //    Console.WriteLine("Generating calls from server");
            //    Directory.CreateDirectory("src/from_server");
            //    using (var modOutput = new FileStream("src/from_server/mod.rs", FileMode.Create))
            //    {
            //        var modSource = new SourceGenerator(false);

            //        foreach (var call in idl.FromServer)
            //        {
            //            var codeSource = new SourceGenerator(true);
            //            CallGenerator.GenerateCall(codeSource, call.Value, message_id);
            //            using (var output = new FileStream($"src/from_server/{call.Key}.rs", FileMode.Create))
            //            {
            //                using (var writer = new StreamWriter(output, leaveOpen: true))
            //                {
            //                    writer.WriteLine(codeSource.GetSource(hasEnums));
            //                }
            //            }
            //            message_id++;

            //            modSource.AddLine($"pub mod {call.Key};");
            //            modSource.AddLine($"pub use {call.Key}::*;");
            //        }

            //        using (var writer = new StreamWriter(modOutput, leaveOpen: true))
            //        {
            //            writer.WriteLine(modSource.GetSource(hasEnums));
            //        }
            //    }
            //}

            Console.WriteLine("Generating library");
            using (var output = new FileStream("src/lib.rs", FileMode.Create))
            {
                var source = new SourceGenerator(false);

                if (idl.EnumLists.Count > 0)
                {
                    source.AddLine("mod enums;");
                    source.AddLine("pub use enums::*;");
                }
                if (idl.Types.Count > 0)
                {
                    source.AddLine("mod types;");
                    source.AddLine("pub use types::*;");
                }
                if (idl.FromClient.Count > 0)
                {
                    source.AddLine("mod from_client;");
                    source.AddLine("pub use from_client::*;");
                }
                if (idl.FromServer.Count > 0)
                {
                    source.AddLine("mod from_server;");
                    source.AddLine("pub use from_server::*;");
                }

                using (var writer = new StreamWriter(output, leaveOpen: true))
                {
                    writer.WriteLine(source.GetSource(hasEnums));
                }
            }
        }
    }
}
