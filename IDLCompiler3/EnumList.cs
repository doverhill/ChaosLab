using System;
using System.Collections.Generic;
using System.Linq;

namespace IDLCompiler
{
    public class EnumList
    {
        public class Option
        {
            public string OwningTypeName;
            public IDLField.FieldType Type;
            public CasedString Name;
            public IDLType CustomType;

            public Option(string owningTypeName, IDLField.FieldType type, IDLType customType)
            {
                OwningTypeName = owningTypeName;
                Type = type;
                CustomType = customType;

                if (Type == IDLField.FieldType.None) Name = CasedString.FromPascal("TypeNone");
                else if (Type == IDLField.FieldType.CustomType) Name = CasedString.FromPascal($"Type{customType.Name}");
                else Name = CasedString.FromPascal($"Type{Type}");
            }

            public static Option FromPascalString(string name, string owningTypeName)
            {
                var casedName = CasedString.FromPascal(name);
                return new Option(owningTypeName, IDLField.FieldType.None, null)
                { 
                    Name = casedName 
                };
            }

            internal string ToEnumDeclarationString()
            {
                if (Type == IDLField.FieldType.None) return Name.ToPascal();
                else if (Type == IDLField.FieldType.CustomType) return $"{Name.ToPascal()}({CustomType.Name})";
                else return $"{Name.ToPascal()}({Type})";
            }

            internal object ToEnumStructMatchString()
            {
                return $"{OwningTypeName}StructTag::{Name.ToPascal()}";
            }

            internal object ToMatchString()
            {
                if (Type == IDLField.FieldType.None) return $"{OwningTypeName}Enum::{Name.ToPascal()}";
                else return $"{OwningTypeName}Enum::{Name.ToPascal()}(value)";
            }

            internal string ToTagEnumDeclarationString()
            {
                return Name.ToPascal();
            }

            internal string ToUnionDeclarationString()
            {
                var fieldName = GetPayloadUnionFieldName(Name);
                if (Type == IDLField.FieldType.None) return $"{fieldName}: [u8; 0],";
                else if (Type == IDLField.FieldType.CustomType) return $"{fieldName}: {CustomType.Name},";
                // FIXME: support custom enum and arrays?
                else return $"{fieldName}: {IDLField.GetRustType(Type, CustomType, OwningTypeName, Name.ToPascal(), null, false)},";
            }

            public static string GetPayloadUnionFieldName(CasedString name)
            {
                return $"payload_{name.ToSnake()}";
            }
        }

        public string Name;
        public List<Option> Options;

        public EnumList(string name, List<string> pascalOptions)
        {
            Name = name;
            Options = pascalOptions.Select(o => Option.FromPascalString(o, name)).ToList();
        }
    }
}
