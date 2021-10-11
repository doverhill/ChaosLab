using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Storm
{
    public static class SyscallHelpers
    {
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

        public static void WriteGuid(BinaryWriter writer, Guid? guid)
        {
            if (!guid.HasValue)
            {
                writer.Write(false);
            }
            else
            {
                writer.Write(true);
                writer.Write(guid.Value.ToByteArray());
            }
        }

        public static Guid? ReadGuid(BinaryReader reader)
        {
            var hasGuid = reader.ReadBoolean();
            if (hasGuid)
                return new Guid(reader.ReadBytes(16));
            else
                return null;
        }
    }
}
