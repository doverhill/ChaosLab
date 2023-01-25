using System;
using System.Collections.Generic;
using System.Linq;

namespace IDLCompiler
{
    public class EnumList
    {
        public class Option
        {
            public IDLField.FieldType Type;
            public CasedString Name;
            public IDLType CustomType;

            public Option(IDLField.FieldType type, IDLType customType)
            {
                Type = type;
                CustomType = customType;

                if (Type == IDLField.FieldType.None) Name = CasedString.FromPascal("TypeNone");
                else if (Type == IDLField.FieldType.CustomType) Name = CasedString.FromPascal($"Type{customType.Name}");
                else Name = CasedString.FromPascal($"Type{Type}");
            }

            public static Option FromPascalString(string name)
            {
                var casedName = CasedString.FromPascal(name);
                return new Option(IDLField.FieldType.None, null)
                { 
                    Name = casedName 
                };
            }

            internal string ToEnumDeclarationString()
            {
                if (Type == IDLField.FieldType.None) return Name.ToPascal();
                else if (Type == IDLField.FieldType.CustomType) return $"{Name.ToPascal()}({CustomType.Name})";
                else return $"{Name.ToPascal()}({IDLField.GetRustType(Type, null, null, null, null, false)})";
            }

            internal object ToEnumStructMatchString(string owningTypeName)
            {
                //owningFieldName = CasedString.FromSnake(owningFieldName).ToPascal();
                return $"{owningTypeName}StructTag::{Name.ToPascal()}";
            }

            internal object ToMatchString(string owningTypeName)
            {
                //owningFieldName = CasedString.FromSnake(owningFieldName).ToPascal();
                if (Type == IDLField.FieldType.None) return $"{owningTypeName}::{Name.ToPascal()}";
                else return $"{owningTypeName}::{Name.ToPascal()}(value)";
            }

            internal string ToTagEnumDeclarationString()
            {
                return Name.ToPascal();
            }

            internal string ToUnionDeclarationString(string owningTypeName)
            {
                var fieldName = GetPayloadUnionFieldName(Name);
                if (Type == IDLField.FieldType.None) return $"{fieldName}: [u8; 0],";
                else if (Type == IDLField.FieldType.String) return $"{fieldName}: ManuallyDrop<String>,";
                else if (Type == IDLField.FieldType.CustomType) return $"{fieldName}: ManuallyDrop<{CustomType.Name}>,";
                //else if (Type == IDLField.FieldType.OneOfType) return $"{fieldName}: ManuallyDrop<{}>,";
                // FIXME: support custom enum and arrays?
                else return $"{fieldName}: {IDLField.GetRustType(Type, CustomType, owningTypeName, "XXX", null, false)},";
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
            Options = pascalOptions.Select(Option.FromPascalString).ToList();
        }

        public EnumList(string name, List<Option> options)
        {
            Name = name;
            Options = options;
        }
    }
}
