using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Uuids;

namespace Core
{
    public static class SyscallHelpers
    {
        public static UInt64? ReadOptionalU64(BinaryReader reader)
        {
            var hasValue = reader.ReadBoolean();
            if (hasValue)
            {
                return (UInt64)reader.ReadUInt64();
            }
            else
            {
                return null;
            }
        }

        public static Int32? ReadOptionalI32(BinaryReader reader)
        {
            var hasValue = reader.ReadBoolean();
            if (hasValue)
            {
                return reader.ReadInt32();
            }
            else
            {
                return null;
            }
        }

        public static void WriteText(BinaryWriter writer, string? text)
        {
            if (text == null)
            {
                writer.Write(false);
            }
            else
            {
                writer.Write(true);
                writer.Write(text.Length);
                writer.Write(text.ToCharArray());
            }
        }

        public static string? ReadText(BinaryReader reader)
        {
            var hasText = reader.ReadBoolean();
            if (hasText)
            {
                var textLength = reader.ReadInt32();
                return new string(reader.ReadChars(textLength));
            }
            else
            {
                return null;
            }
        }

        public static void WriteUuid(BinaryWriter writer, Uuid? uuid)
        {
            if (!uuid.HasValue)
            {
                writer.Write(false);
            }
            else
            {
                writer.Write(true);
                writer.Write(uuid.Value.ToByteArray());
            }
        }

        public static Uuid? ReadUuid(BinaryReader reader)
        {
            var hasUuid = reader.ReadBoolean();
            if (hasUuid)
                return new Uuid(reader.ReadBytes(16));
            else
                return null;
        }
    }
}
