using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Storm
{
    internal enum HandleType
    {
        Service,
        Channel
    }

    internal class KernelHandle
    {
        public ulong OwningPID;
        public ulong Id;
        public HandleType HandleType;

        public KernelHandle(ulong id, ulong owningPID, HandleType handleType)
        {
            Id = id;
            OwningPID = owningPID;
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

        public static void CleanupProcess(ulong pid)
        {
            lock (_lock)
            {
                kernelHandles = kernelHandles.Where(h => h.Value.OwningPID != pid).ToDictionary(h => h.Key, h => h.Value);
            }
        }
    }
}
