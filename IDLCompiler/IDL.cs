using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json.Serialization;

namespace IDLCompiler
{
    public class IDL
    {
        [JsonPropertyName("protocol")]
        public IDLProtocol Protocol;
        [JsonPropertyName("enums")]
        public Dictionary<string, List<string>> Enums;
        [JsonPropertyName("types")]
        public Dictionary<string, IDLType> Types;
        [JsonPropertyName("from_client_size")]
        public int FromClientSize = 1024 * 1024;
        [JsonPropertyName("from_client")]
        public Dictionary<string, IDLCall> FromClient;
        [JsonPropertyName("from_server_size")]
        public int FromServerSize = 1024 * 1024;
        [JsonPropertyName("from_server")]
        public Dictionary<string, IDLCall> FromServer;

        public Dictionary<string, EnumList> EnumLists;

        private void ValidateEnumList(string name, List<string> enumList)
        {
            if (string.IsNullOrEmpty(name)) throw new ArgumentException("Enum name is missing");
            if (!CasedString.IsPascal(name)) throw new ArgumentException($"Enum name '{name}' must be pascal case");
            if (enumList.Count == 0) throw new ArgumentException("Enum '{name}' contains no items");
            foreach (var item in enumList)
            {
                if (!CasedString.IsPascal(item)) throw new ArgumentException($"Enum item '{item}' for enum '{name}' must be pascal case");
            }
        }

        public void Validate()
        {
            if (Protocol == null) throw new ArgumentException("Protocol header is missing");
            Protocol.Validate();

            if (Enums == null) Enums = new();
            foreach (var enumList in Enums)
            {
                ValidateEnumList(enumList.Key, enumList.Value);
            }
            var enums = Enums.ToDictionary(e => e.Key, e => new EnumList(e.Key, e.Value));
            EnumLists = enums;

            if (Types == null) Types = new();
            foreach (var type in Types)
            {
                type.Value.Name = type.Key;
                type.Value.Validate(type.Key, enums, Types);
            }

            //if (FromClientSize == 0) FromClientSize = 4096;

            if (FromClient == null) FromClient = new();
            foreach (var call in FromClient)
            {
                call.Value.Validate(call.Key, enums, Types);
            }

            //if (FromServerSize == 0) FromServerSize = 4096;

            if (FromServer == null) FromServer = new();
            foreach (var call in FromServer)
            {
                call.Value.Validate(call.Key, enums, Types);
            }

            // apply inheritance
            foreach (var type in Types.Values)
            {
                if (!string.IsNullOrEmpty(type.InheritsFrom))
                {
                    if (Types.TryGetValue(type.InheritsFrom, out var customType))
                    {
                        type.Fields = customType.Fields.Values.ToList().Concat(type.Fields.Values).ToDictionary(f => f.Name);
                    }
                    else throw new ArgumentException($"Inherit from type '{type.InheritsFrom}' for '{type.Name}' not recognized as a custom type");
                }
            }
        }

        private string Indent(int level)
        {
            return new string(' ', level * 4);
        }

        private void DumpField(string name, IDLField field, int indent)
        {
            Console.Write(Indent(indent));
            Console.ForegroundColor = ConsoleColor.Yellow;
            Console.Write($"{name}");
            Console.ForegroundColor = ConsoleColor.Gray;
            Console.WriteLine($": Type={field.Type}, IsArray={field.IsArray}, CustomType={field.CustomType?.Name}, CustomEnumList={field.CustomEnumList?.Name}, OneOf={field.CustomOneOfOptions?.Count}, ");
        }

        public void Dump()
        {
            Console.WriteLine($"PROTOCOL '{Protocol.Name}' version {Protocol.Version}:");
            Console.WriteLine(Indent(1) + "ENUMS:");
            foreach (var enumList in Enums)
            {
                Console.Write(Indent(2));
                Console.ForegroundColor = ConsoleColor.DarkGreen;
                Console.Write($"{enumList.Key}");
                Console.ForegroundColor = ConsoleColor.Gray;
                Console.WriteLine($": Options=[{string.Join(", ", enumList.Value)}]");
            }
            Console.WriteLine(Indent(1) + "TYPES:");
            foreach (var type in Types)
            {
                Console.Write(Indent(2));
                Console.ForegroundColor = ConsoleColor.Green;
                Console.Write($"{type.Key}");
                Console.ForegroundColor = ConsoleColor.Gray;
                Console.WriteLine($": InheritsFrom='{type.Value.InheritsFrom}'");
                Console.WriteLine(Indent(3) + "FIELDS:");
                foreach (var field in type.Value.Fields)
                {
                    DumpField(field.Key, field.Value, 4);
                }
            }
            Console.WriteLine(Indent(1) + "FROM CLIENT:");
            foreach (var call in FromClient)
            {
                Console.Write(Indent(2));
                Console.ForegroundColor = ConsoleColor.Blue;
                Console.Write($"{call.Key}");
                Console.ForegroundColor = ConsoleColor.Gray;
                Console.WriteLine($": Coalesce={call.Value.Coalesce}");
                Console.WriteLine(Indent(3) + "PARAMETERS:");
                foreach (var p in call.Value.Parameters)
                {
                    DumpField(p.Key, p.Value, 4);
                }
                Console.WriteLine(Indent(3) + "RETURNS:");
                foreach (var p in call.Value.ReturnValues)
                {
                    DumpField(p.Key, p.Value, 4);
                }
            }
            Console.WriteLine(Indent(1) + "FROM SERVER:");
            foreach (var call in FromServer)
            {
                Console.Write(Indent(2));
                Console.ForegroundColor = ConsoleColor.Blue;
                Console.Write($"{call.Key}");
                Console.ForegroundColor = ConsoleColor.Gray;
                Console.WriteLine($": Coalesce={call.Value.Coalesce}");
                Console.WriteLine(Indent(3) + "PARAMETERS:");
                foreach (var p in call.Value.Parameters)
                {
                    DumpField(p.Key, p.Value, 4);
                }
                Console.WriteLine(Indent(3) + "RETURNS:");
                foreach (var p in call.Value.ReturnValues)
                {
                    DumpField(p.Key, p.Value, 4);
                }
            }
        }
    }
}
