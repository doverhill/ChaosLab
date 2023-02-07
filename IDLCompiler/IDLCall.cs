using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json.Serialization;

namespace IDLCompiler
{
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

        public string Name;
        public CallType Type;

        public void Validate(string name, Dictionary<string, EnumList> customEnumLists, Dictionary<string, IDLType> customTypes)
        {
            if (string.IsNullOrEmpty(name)) throw new ArgumentNullException("Field name is missing");
            if (!CasedString.IsSnake(name)) throw new ArgumentException($"Field name '{name}' must be snake case");

            Name = name;

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

        public (IDLType, string) ToParametersType()
        {
            var casedString = CasedString.FromSnake(Name);
            var name = casedString.ToPascal() + "Parameters";
            var messageName = casedString.ToScreamingSnake() + "_PARAMETERS";

            if (Parameters.Count > 0)
            {
                return (new IDLType
                {
                    Name = name,
                    Fields = Parameters
                }, messageName);
            }
            return (null, messageName);
        }

        public (IDLType, string) ToReturnsType(bool fromServer)
        {
            if (fromServer)
            {
                if (Type != CallType.Event && Type != CallType.SingleEvent) throw new Exception($"{Name}: Only event types are supported in server->client calls");
                if (ReturnValues.Count > 0) throw new Exception($"{Name}: Return values not allowed in server->client events");
            }

            var casedString = CasedString.FromSnake(Name);
            var name = casedString.ToPascal() + "Returns";
            var messageName = casedString.ToScreamingSnake() + "_RETURNS";

            if (ReturnValues.Count > 0)
            {
                return (new IDLType
                {
                    Name = name,
                    Fields = ReturnValues
                }, messageName);
            }
            return (null, messageName);
        }
    }
}
