using System;
using System.Collections.Generic;
using System.Data.Common;
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

            var hasEnums = idl.EnumLists.Count > 0;
            var hasTypes = idl.Types.Count > 0;

            if (idl.EnumLists.Count > 0)
            {
                Console.WriteLine("Generating enums");
                Directory.CreateDirectory("src/enums");
                using (var modOutput = new FileStream("src/enums/mod.rs", FileMode.Create))
                {
                    var modSource = new SourceGenerator(false);

                    foreach (var enumList in idl.EnumLists)
                    {
                        var enumName = CasedString.FromPascal(enumList.Key);

                        var codeSource = new SourceGenerator(true);
                        EnumGenerator.GenerateEnum(codeSource, enumList.Value, false);
                        using (var output = new FileStream($"src/enums/{enumName.ToSnake()}.rs", FileMode.Create))
                        {
                            using (var writer = new StreamWriter(output, leaveOpen: true))
                            {
                                writer.WriteLine(codeSource.GetSource(hasTypes, hasEnums));
                            }
                        }

                        modSource.AddLine($"mod {enumName.ToSnake()};");
                        modSource.AddLine($"pub use {enumName.ToSnake()}::*;");
                    }

                    using (var writer = new StreamWriter(modOutput, leaveOpen: true))
                    {
                        writer.WriteLine(modSource.GetSource(hasTypes, hasEnums));
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
                        var typeName = CasedString.FromPascal(type.Key);

                        var codeSource = new SourceGenerator(true);
                        TypeGenerator.GenerateType(codeSource, type.Value);
                        using (var output = new FileStream($"src/types/{typeName.ToSnake()}.rs", FileMode.Create))
                        {
                            using (var writer = new StreamWriter(output, leaveOpen: true))
                            {
                                writer.WriteLine(codeSource.GetSource(hasTypes, hasEnums));
                            }
                        }

                        modSource.AddLine($"mod {typeName.ToSnake()};");
                        modSource.AddLine($"pub use {typeName.ToSnake()}::*;");
                    }

                    using (var writer = new StreamWriter(modOutput, leaveOpen: true))
                    {
                        writer.WriteLine(modSource.GetSource(hasTypes, hasEnums));
                    }
                }
            }

            //var messageId = 1;
            var messageIds = new List<string>();

            if (idl.FromClient.Count > 0)
            {
                Console.WriteLine("Generating calls from client");
                Directory.CreateDirectory("src/from_client");
                using (var modOutput = new FileStream("src/from_client/mod.rs", FileMode.Create))
                {
                    var modSource = new SourceGenerator(false);

                    foreach (var call in idl.FromClient)
                    {
                        var codeSource = new SourceGenerator(true);
                        //CallGenerator.GenerateCall(codeSource, call.Value, message_id);

                        var (parametersType, parametersMessageName) = call.Value.ToParametersType();
                        messageIds.Add(parametersMessageName);
                        if (parametersType != null)
                        {
                            TypeGenerator.GenerateType(codeSource, parametersType);
                        }
                        var (returnsType, returnsMessageName) = call.Value.ToReturnsType(false);
                        if (returnsType != null)
                        {
                            messageIds.Add(returnsMessageName);
                            TypeGenerator.GenerateType(codeSource, returnsType);
                        }

                        using (var output = new FileStream($"src/from_client/{call.Key}.rs", FileMode.Create))
                        {
                            using (var writer = new StreamWriter(output, leaveOpen: true))
                            {
                                writer.WriteLine(codeSource.GetSource(hasTypes, hasEnums));
                            }
                        }

                        modSource.AddLine($"mod {call.Key};");
                        modSource.AddLine($"pub use {call.Key}::*;");
                    }

                    using (var writer = new StreamWriter(modOutput, leaveOpen: true))
                    {
                        writer.WriteLine(modSource.GetSource(hasTypes, hasEnums));
                    }
                }
            }

            if (idl.FromClient.Count > 0)
            {
                Console.WriteLine("Generating calls from server");
                Directory.CreateDirectory("src/from_server");
                using (var modOutput = new FileStream("src/from_server/mod.rs", FileMode.Create))
                {
                    var modSource = new SourceGenerator(false);

                    foreach (var call in idl.FromServer)
                    {
                        var codeSource = new SourceGenerator(true);
                        //CallGenerator.GenerateCall(codeSource, call.Value, message_id);

                        var (parametersType, parametersMessageName) = call.Value.ToParametersType();
                        messageIds.Add(parametersMessageName);
                        if (parametersType != null)
                        {
                            TypeGenerator.GenerateType(codeSource, parametersType);
                        }
                        var (returnsType, returnsMessageName) = call.Value.ToReturnsType(true);
                        if (returnsType != null)
                        {
                            messageIds.Add(returnsMessageName);
                            TypeGenerator.GenerateType(codeSource, returnsType);
                        }

                        using (var output = new FileStream($"src/from_server/{call.Key}.rs", FileMode.Create))
                        {
                            using (var writer = new StreamWriter(output, leaveOpen: true))
                            {
                                writer.WriteLine(codeSource.GetSource(hasTypes, hasEnums));
                            }
                        }

                        modSource.AddLine($"mod {call.Key};");
                        modSource.AddLine($"pub use {call.Key}::*;");
                    }

                    using (var writer = new StreamWriter(modOutput, leaveOpen: true))
                    {
                        writer.WriteLine(modSource.GetSource(hasTypes, hasEnums));
                    }
                }
            }

            Console.WriteLine("Generating library scaffolding");
            Directory.CreateDirectory("src/code");
            if (!File.Exists("src/code/mod.rs"))
            {
                using (var writer = File.CreateText("src/code/mod.rs"))
                {
                    writer.WriteLine("// library code goes in this mod...");
                }
            }

            Console.WriteLine("Generting channel");
            using (var output = new FileStream("src/channel.rs", FileMode.Create))
            {
                var source = new SourceGenerator(true);

                ChannelGenerator.GenerateChannel(source, idl);
                using (var writer = new StreamWriter(output, leaveOpen: true))
                {
                    writer.WriteLine(source.GetSource(hasTypes, hasEnums));
                }
            }

            Console.WriteLine("Generating server");
            using (var output = new FileStream("src/server.rs", FileMode.Create))
            {
                var source = new SourceGenerator(true);

                ClientServerGenerator.GenerateServer(source, idl, idl.FromClient, idl.FromServer);
                using (var writer = new StreamWriter(output, leaveOpen: true))
                {
                    writer.WriteLine(source.GetSource(hasTypes, hasEnums));
                }
            }

            Console.WriteLine("Generating client");
            using (var output = new FileStream("src/client.rs", FileMode.Create))
            {
                var source = new SourceGenerator(true);

                ClientServerGenerator.GenerateClient(source, idl, idl.FromServer, idl.FromClient, idl.FromServerSize, idl.FromClientSize);
                using (var writer = new StreamWriter(output, leaveOpen: true))
                {
                    writer.WriteLine(source.GetSource(hasTypes, hasEnums));
                }
            }

            Console.WriteLine("Generating messages");
            using (var output = new FileStream("src/message_ids.rs", FileMode.Create))
            {
                var source = new SourceGenerator(false);

                var enumBlock = source.AddBlock("pub enum MessageIds");
                foreach (var message in messageIds)
                {
                    enumBlock.AddLine(message + ",");
                }

                using (var writer = new StreamWriter(output, leaveOpen: true))
                {
                    writer.WriteLine(source.GetSource(false, false));
                }
            }

            Console.WriteLine("Generating library");
            using (var output = new FileStream("src/lib.rs", FileMode.Create))
            {
                var source = new SourceGenerator(false);

                //source.AddLine("#![no_std]");
                source.AddLine("extern crate alloc;");
                source.AddLine("extern crate library_chaos;");
                source.AddBlank();

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

                source.AddLine("mod message_ids;");
                source.AddLine("pub use message_ids::MessageIds;");

                source.AddLine("mod channel;");
                source.AddLine("pub use channel::*;");

                source.AddLine("mod server;");
                source.AddLine("pub use server::*;");
                source.AddLine("mod client;");
                source.AddLine("pub use client::*;");

                source.AddLine("mod code;");
                source.AddLine("pub use code::*;");

                var protocolName = CasedString.FromSnake(idl.Protocol.Name);
                source.AddLine($"pub static {protocolName.ToScreamingSnake()}_PROTOCOL_NAME: &str = \"{idl.Protocol.Name}\";");

                using (var writer = new StreamWriter(output, leaveOpen: true))
                {
                    writer.WriteLine(source.GetSource(hasTypes, hasEnums));
                }
            }
        }
    }
}
