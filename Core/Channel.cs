using System;
using System.Collections.Generic;
using System.IO.MemoryMappedFiles;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Core
{
    public class Channel
    {
        private Handle channelHandle;
        private MemoryMappedViewAccessor accessor;

        public Channel(Handle channelHandle)
        {
            this.channelHandle = channelHandle;

            var memoryName = GetMapName(channelHandle);
            var mapSize = 4096;
            var sharedMemory = MemoryMappedFile.CreateOrOpen(memoryName, mapSize, MemoryMappedFileAccess.ReadWrite);
            accessor = sharedMemory.CreateViewAccessor();
        }

        private string GetMapName(Handle channelHandle)
        {
            var mapName = "__chaos_channel_" + channelHandle.Id;
            return mapName;
        }

        public unsafe byte* GetRawPointer()
        {
            byte* raw = null;
            accessor.SafeMemoryMappedViewHandle.AcquirePointer(ref raw);
            return raw;
        }

        public override string ToString()
        {
            unsafe
            {
                return "[CHANNEL: handle=" + channelHandle.ToString() + ", pointer=" + (ulong)GetRawPointer() + "]";
            }
        }
    }
}
