﻿using System;
using System.Collections.Generic;
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
                parameter.Value.Validate($"{Name}Parameters", parameter.Key, customEnumLists, customTypes);
            }

            if (ReturnValues == null) ReturnValues = new();
            foreach (var returnValue in ReturnValues)
            {
                returnValue.Value.Validate($"{Name}Returns", returnValue.Key, customEnumLists, customTypes);
            }
        }
    }
}
