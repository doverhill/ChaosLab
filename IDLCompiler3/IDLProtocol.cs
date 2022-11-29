using System;
using System.Text.Json.Serialization;

namespace IDLCompiler
{
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
}
