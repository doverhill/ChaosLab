namespace IDLCompiler
{
    internal class Field
    {
        public enum DataType
        {
            String,
            Signed32,
            Unsigned32,
            Signed64,
            Unsigned64,
            UnsignedNative,
            SignedNative,
            Float32,
            Float64,
            Boolean,
            DateTime,
            Date,
            Time,
            Byte,
            Type,
            Enum
        }

        public DataType Type;
        //public bool IsArray;
        //public int ArrayLength;
        //public int Capacity;
        public CasedString TypeName;
        public CasedString Name;

        public Field(string fieldDescription, List<IDLType> types)
        {
            var parts = fieldDescription.Split(" ");
            if (parts.Length == 0 || parts.Length > 2) throw new Exception("Malformed type and name: '" + fieldDescription + "'");

            var typeDescription = parts[0];

            // parse type
            if (typeDescription == "string") Type = DataType.String;
            else if (typeDescription == "i32") Type = DataType.Signed32;
            else if (typeDescription == "u32") Type = DataType.Unsigned32;
            else if (typeDescription == "i64") Type = DataType.Signed64;
            else if (typeDescription == "u64") Type = DataType.Unsigned64;
            else if (typeDescription == "isize") Type = DataType.SignedNative;
            else if (typeDescription == "usize") Type = DataType.UnsignedNative;
            else if (typeDescription == "f32") Type = DataType.Float32;
            else if (typeDescription == "f64") Type = DataType.Float64;
            else if (typeDescription == "bool") Type = DataType.Boolean;
            else if (typeDescription == "datetime") Type = DataType.DateTime;
            else if (typeDescription == "date") Type = DataType.Date;
            else if (typeDescription == "time") Type = DataType.Time;
            else if (typeDescription == "byte") Type = DataType.Byte;
            else
            {
                // see if there is a type for this
                var foundType = false;
                foreach (var otherType in types)
                {
                    if (otherType.Name == typeDescription)
                    {
                        foundType = true;
                    }
                }
                if (!foundType) throw new Exception("Malformed type part: '" + typeDescription + "'");
                Type = DataType.Type;
                TypeName = CasedString.FromPascal(typeDescription);
            }

            if (parts.Length == 2)
            {
                var fieldName = parts[1];
                Name = CasedString.FromPascal(fieldName);
            }
        }

        public string GetStructType()
        {
            if (Type == DataType.String) return "String";
            else if (Type == DataType.Signed32) return "i32";
            else if (Type == DataType.Unsigned32) return "u32";
            else if (Type == DataType.Signed64) return "i64";
            else if (Type == DataType.Unsigned64) return "u64";
            else if (Type == DataType.SignedNative) return "isize";
            else if (Type == DataType.UnsignedNative) return "usize";
            else if (Type == DataType.Float32) return "f32";
            else if (Type == DataType.Float64) return "f64";
            else if (Type == DataType.Boolean) return "bool";
            else if (Type == DataType.DateTime) return "i32";
            else if (Type == DataType.Date) return "i32";
            else if (Type == DataType.Time) return "i32";
            else if (Type == DataType.Byte) return "u8";
            else if (Type == DataType.Type) return TypeName.ToPascal();

            throw new Exception("Unknown field type");
        }

        public string GetCallType()
        {
            if (Type == DataType.String) return "&str";
            else if (Type == DataType.Signed32) return "i32";
            else if (Type == DataType.Unsigned32) return "u32";
            else if (Type == DataType.Signed64) return "i64";
            else if (Type == DataType.Unsigned64) return "u64";
            else if (Type == DataType.SignedNative) return "isize";
            else if (Type == DataType.UnsignedNative) return "usize";
            else if (Type == DataType.Float32) return "f32";
            else if (Type == DataType.Float64) return "f64";
            else if (Type == DataType.Boolean) return "bool";
            else if (Type == DataType.DateTime) return "i32";
            else if (Type == DataType.Date) return "i32";
            else if (Type == DataType.Time) return "i32";
            else if (Type == DataType.Byte) return "u8";
            else if (Type == DataType.Type) return TypeName.ToPascal();

            throw new Exception("Unknown field type");
        }
    }
}
