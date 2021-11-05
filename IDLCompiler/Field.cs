using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

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
            Float32,
            Float64,
            Boolean,
            DateTime,
            Date,
            Time,
            Byte
        }

        public DataType Type;
        public bool IsArray;
        public int ArrayLength;
        public int Capacity;

        public Field(string fieldDescription)
        {
            // format is
            // string(100) FieldName
            // string(100) FieldNames[3]
            // u64 Count
            // u64 Counts[4]


        }
    }
}
