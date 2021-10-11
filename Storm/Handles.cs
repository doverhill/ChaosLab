using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Storm
{
    internal enum HandleType
    {
        Service
    }

    internal class KernelHandle
    {
        public ulong Id;
        public ulong PID;
        public HandleType HandleType;

        public KernelHandle(ulong id, ulong pid, HandleType handleType)
        {
            Id = id;
            PID = pid;
            HandleType = handleType;
        }
    }

    internal static class Handles
    {
        private static object _lock = new object();
        private static ulong nextId = 1;
        private static Dictionary<ulong, KernelHandle> kernelHandles = new Dictionary<ulong, KernelHandle>();

        public static ulong AllocateHandle(ulong pid, HandleType type)
        {
            lock (_lock)
            {
                var id = nextId++;
                var handle = new KernelHandle(id, pid, type);
                kernelHandles.Add(id, handle);
                return handle.Id;
            }
        }
    }
}
