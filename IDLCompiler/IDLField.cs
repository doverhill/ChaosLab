using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace IDLCompiler
{
    public class IDLField
    {
        [JsonPropertyName("type")]
        public string NamedType;
        [JsonPropertyName("array")]
        public bool IsArray;

        public string Name;

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

        //public class OneOfOption
        //{
        //    public FieldType Type;
        //    public IDLType CustomType;

        //    public OneOfOption(FieldType type, IDLType customType)
        //    {
        //        Type = type;
        //        CustomType = customType;
        //    }
        //}

        public FieldType Type;
        public IDLType CustomType = null;
        public EnumList CustomEnumList = null;
        public List<EnumList.Option> CustomOneOfOptions = null;

        public static string GetOneOfTypeName(string owningTypeName, string fieldName)
        {
            //if (fieldType != FieldType.OneOfType) throw new ArgumentException($"Can not get OneOfTypeName for field of type {fieldType}");
            return $"{owningTypeName}{CasedString.FromSnake(fieldName).ToPascal()}Enum";
        }

        public static string GetRustType(FieldType fieldType, IDLType customType, string owningTypeName, string fieldName, EnumList customEnumList, bool isArray)
        {
            var typeName = fieldType switch
            {
                FieldType.None => throw new ArgumentException("Can not get rust type of type None"),
                FieldType.U8 => "u8",
                FieldType.U64 => "u64",
                FieldType.I64 => "i64",
                FieldType.Bool => "bool",
                FieldType.String => "String",
                FieldType.CustomType => customType.Name,
                FieldType.OneOfType => GetOneOfTypeName(owningTypeName, fieldName),
                FieldType.Enum => customEnumList.Name
            };

            if (isArray) return $"Vec<{typeName}>";
            return typeName;
        }

        //public string GetRustType(string owningTypeName, bool useStr, bool allowMutPointer)
        //{
        //    return GetRustType(Type, CustomType, Name, CustomEnumList, IsArray, owningTypeName, useStr, allowMutPointer);
        //}

        //public string GetInnerRustType(string owningTypeName, bool useStr)
        //{
        //    return GetInnerRustType(Type, CustomType, Name, CustomEnumList, IsArray, owningTypeName, useStr);
        //}

        //public static string GetRustType(FieldType type, IDLType customType, string fieldName, EnumList customEnumList, bool isArray, string owningTypeName, bool useStr, bool allowMutPointer)
        //{
        //    var typeName = type switch
        //    {
        //        FieldType.None => throw new ArgumentException("Can not get rust type of type None"),
        //        FieldType.U8 => "u8",
        //        FieldType.U64 => "u64",
        //        FieldType.I64 => "i64",
        //        FieldType.Bool => "bool",
        //        FieldType.String => useStr ? "&str" : "String",
        //        FieldType.CustomType => customType.Name,
        //        FieldType.OneOfType => $"{owningTypeName}{CasedString.FromSnake(fieldName).ToPascal()}Enum",
        //        FieldType.Enum => customEnumList.Name
        //    };

        //    var mutPointer = type == FieldType.CustomType && allowMutPointer ? "*mut " : "";
        //    if (isArray) return $"Vec<{mutPointer}{typeName}>";
        //    return typeName;
        //}

        //public static string GetInnerRustType(FieldType type, IDLType customType, string fieldName, EnumList customEnumList, bool isArray, string owningTypeName, bool useStr)
        //{
        //    if (!isArray) throw new ArgumentException("GetInnerRustType called on non-array");

        //    var typeName = type switch
        //    {
        //        FieldType.None => throw new ArgumentException("Can not get rust type of type None"),
        //        FieldType.U8 => "u8",
        //        FieldType.U64 => "u64",
        //        FieldType.I64 => "i64",
        //        FieldType.Bool => "bool",
        //        FieldType.String => useStr ? "&str" : "String",
        //        FieldType.CustomType => customType.Name,
        //        FieldType.OneOfType => $"{owningTypeName}{CasedString.FromSnake(fieldName).ToPascal()}Enum",
        //        FieldType.Enum => customEnumList.Name
        //    };

        //    return typeName;
        //}

        public void Validate(string name, Dictionary<string, EnumList> customEnumLists, Dictionary<string, IDLType> customTypes)
        {
            if (string.IsNullOrEmpty(name)) throw new ArgumentNullException("Field name is missing");
            if (!CasedString.IsSnake(name)) throw new ArgumentException($"Field name '{name}' must be snake case");

            Name = name;

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
                    CustomOneOfOptions = new List<EnumList.Option>();
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
                                CustomOneOfOptions.Add(new EnumList.Option(oneOfType, customType));
                            }
                            else throw new ArgumentException($"Custom type '{option}' not found for one-of list on field '{name}'");
                        }
                        else
                        {
                            CustomOneOfOptions.Add(new EnumList.Option(oneOfType, null));
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

        //internal IDLField Clone()
        //{
        //    return new IDLField
        //    {
        //        CustomEnumList = CustomEnumList,
        //        CustomOneOfOptions = CustomOneOfOptions,
        //        CustomType = CustomType,
        //        IsArray = IsArray,
        //        Name = Name,
        //        NamedType = NamedType,
        //        Type = Type
        //    };
        //}
    }
}
