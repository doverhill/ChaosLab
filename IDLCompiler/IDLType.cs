using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace IDLCompiler
{
    public class IDLType
    {
        public string Name;

        [JsonPropertyName("inherits_from")]
        public string InheritsFrom;
        [JsonPropertyName("fields")]
        public Dictionary<string, IDLField> Fields;

        //private IDLType _inheritsFrom = null;

        //public IDLType GetInheritsFrom() => _inheritsFrom;

        public void Validate(string name, Dictionary<string, EnumList> customEnumLists, Dictionary<string, IDLType> customTypes)
        {
            if (string.IsNullOrEmpty(name)) throw new ArgumentNullException("Type name is missing");
            if (!CasedString.IsPascal(name)) throw new ArgumentException($"Type name '{name}' must be pascal case");

            if (Fields == null) Fields = new();
            foreach (var field in Fields)
            {
                field.Value.Validate(field.Key, customEnumLists, customTypes);
            }
        }
    }
}
