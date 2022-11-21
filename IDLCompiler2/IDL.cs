using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json.Serialization;

namespace IDLCompiler
{
    public class EnumList
    {
        public string Name;
        public List<string> Options;

        public EnumList(string name, List<string> options)
        {
            Name = name;
            Options = options;
        }
    }

    public class IDL
    {
        [JsonPropertyName("protocol")]
        public IDLProtocol Protocol;
        [JsonPropertyName("enums")]
        public Dictionary<string, List<string>> Enums;
        [JsonPropertyName("types")]
        public Dictionary<string, IDLType> Types;
        [JsonPropertyName("from_client")]
        public Dictionary<string, IDLCall> FromClient;
        [JsonPropertyName("from_server")]
        public Dictionary<string, IDLCall> FromServer;

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

            if (Types == null) Types = new();
            foreach (var type in Types)
            {
                type.Value.Name = type.Key;
                type.Value.Validate(type.Key, enums, Types);
            }

            if (FromClient == null) FromClient = new();
            foreach (var call in FromClient)
            {
                call.Value.Validate(call.Key, enums, Types);
            }

            if (FromServer == null) FromServer = new();
            foreach (var call in FromServer)
            {
                call.Value.Validate(call.Key, enums, Types);
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
                Console.WriteLine($": Type={call.Value.Type}");
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
                Console.WriteLine($": Type={call.Value.Type}");
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

    public class IDLProtocol
    {
        [JsonPropertyName("name")]
        public string Name;
        [JsonPropertyName("version")]
        public int Version;

        public void Validate()
        {
            if (string.IsNullOrEmpty(Name)) throw new ArgumentNullException("Protocol name is missing");
            if (Name.Length > 32) throw new ArgumentException("Protocol name is too long (max 32)");
            if (!CasedString.IsSnake(Name)) throw new ArgumentException($"Protocol name '{Name}' must be snake case");
            if (Version < 1) throw new ArgumentException("Protocol version needs to be at least 1");
        }
    }

    public class IDLType
    {
        public string Name;

        [JsonPropertyName("inherits_from")]
        public string InheritsFrom;
        [JsonPropertyName("fields")]
        public Dictionary<string, IDLField> Fields;

        private IDLType _inheritsFrom = null;

        public void Validate(string name, Dictionary<string, EnumList> customEnumLists, Dictionary<string, IDLType> customTypes)
        {
            if (string.IsNullOrEmpty(name)) throw new ArgumentNullException("Type name is missing");
            if (!CasedString.IsPascal(name)) throw new ArgumentException($"Type name '{name}' must be pascal case");

            if (!string.IsNullOrEmpty(InheritsFrom))
            {
                if (customTypes.TryGetValue(InheritsFrom, out var customType))
                {
                    _inheritsFrom = customType;
                }
                else throw new ArgumentException($"Inherit from type '{InheritsFrom}' for '{name}' not recognized as a custom type");
            }

            foreach (var field in Fields)
            {
                field.Value.Validate(field.Key, customEnumLists, customTypes);
            }
        }
    }

    public class IDLField
    {
        [JsonPropertyName("type")]
        public string NamedType;
        [JsonPropertyName("array")]
        public bool IsArray;

        public enum FieldType
        {
            None,
            U8,
            U64,
            I64,
            Bool,
            String,
            CustomType,
            OneOfType,
            Enum
        }

        public class OneOfOption
        {
            public FieldType Type;
            public IDLType CustomType;

            public OneOfOption(FieldType type, IDLType customType)
            {
                Type = type;
                CustomType = customType;
            }
        }

        public FieldType Type;
        public IDLType CustomType = null;
        public EnumList CustomEnumList = null;
        public List<OneOfOption> CustomOneOfOptions = null;

        public void Validate(string name, Dictionary<string, EnumList> customEnumLists, Dictionary<string, IDLType> customTypes)
        {
            if (string.IsNullOrEmpty(name)) throw new ArgumentNullException("Field name is missing");
            if (!CasedString.IsSnake(name)) throw new ArgumentException($"Field name '{name}' must be snake case");

            Type = NamedType switch
            {
                "u8" => FieldType.U8,
                "u64" => FieldType.U64,
                "i64" => FieldType.I64,
                "bool" => FieldType.Bool,
                "string" => FieldType.String,
                _ => FieldType.CustomType
            };

            if (Type == FieldType.CustomType)
            {
                if (NamedType.Contains('|'))
                {
                    // "one of" list of types
                    Type = FieldType.OneOfType;
                    CustomOneOfOptions = new List<OneOfOption>();
                    foreach (var option in NamedType.Split('|'))
                    {
                        var oneOfType = option switch
                        {
                            "none" => FieldType.None,
                            "u8" => FieldType.U8,
                            "u64" => FieldType.U64,
                            "i64" => FieldType.I64,
                            "bool" => FieldType.Bool,
                            "string" => FieldType.String,
                            _ => FieldType.CustomType
                        };

                        if (oneOfType == FieldType.CustomType)
                        {
                            if (customTypes.TryGetValue(option, out var customType))
                            {
                                CustomOneOfOptions.Add(new OneOfOption(oneOfType, customType));
                            }
                            else throw new ArgumentException($"Custom type '{option}' not found for one-of list on field '{name}'");
                        }
                        else
                        {
                            CustomOneOfOptions.Add(new OneOfOption(oneOfType, null));
                        }
                    }
                }
                else
                {
                    if (customTypes.TryGetValue(NamedType, out var customType))
                    {
                        CustomType = customType;
                    }
                    else if (customEnumLists.TryGetValue(NamedType, out var customEnumList))
                    {
                        Type = FieldType.Enum;
                        CustomEnumList = customEnumList;
                    }
                    else throw new ArgumentException($"Type '{NamedType}' for '{name}' not recognized as a builtin or custom type");
                }
            }
        }
    }

    public class IDLCall
    {
        public enum CallType
        {
            Event,
            SingleEvent,
            Call
        }

        [JsonPropertyName("type")]
        public string NamedType;
        [JsonPropertyName("parameters")]
        public Dictionary<string, IDLField> Parameters;
        [JsonPropertyName("returns")]
        public Dictionary<string, IDLField> ReturnValues;

        public CallType Type;

        public void Validate(string name, Dictionary<string, EnumList> customEnumLists, Dictionary<string, IDLType> customTypes)
        {
            if (string.IsNullOrEmpty(name)) throw new ArgumentNullException("Field name is missing");
            if (!CasedString.IsSnake(name)) throw new ArgumentException($"Field name '{name}' must be snake case");

            if (string.IsNullOrEmpty(NamedType)) throw new ArgumentException($"Type for call '{name}' is missing");
            Type = NamedType switch
            {
                "event" => CallType.Event,
                "single_event" => CallType.SingleEvent,
                "call" => CallType.Call,
                _ => throw new ArgumentException($"Unknown call type '{NamedType}' for call '{name}'")
            };

            if (Parameters == null) Parameters = new();
            foreach (var parameter in Parameters)
            {
                parameter.Value.Validate(parameter.Key, customEnumLists, customTypes);
            }

            if (ReturnValues == null) ReturnValues = new();
            foreach (var returnValue in ReturnValues)
            {
                returnValue.Value.Validate(returnValue.Key, customEnumLists, customTypes);
            }
        }
    }
}
