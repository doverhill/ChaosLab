using Core;
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
        public ulong Id;
        public HashSet<ulong> OwningPIDs;
        public HandleType Type;

        public KernelHandle(ulong id, ulong owningPID, HandleType type)
        {
            Id = id;
            OwningPIDs = new HashSet<ulong> { owningPID };
            Type = type;
        }

        public KernelHandle(ulong id, ulong owningPID, ulong additionalPID, HandleType type)
        {
            Id = id;
            OwningPIDs = new HashSet<ulong> { owningPID, additionalPID };
            Type = type;
        }

        public ulong GetOtherPID(ulong PID)
        {
            if (OwningPIDs.Contains(PID) && OwningPIDs.Count == 2)
            {
                return OwningPIDs.Where(p => PID != p).First();
            }
            return Handle.None;
        }
    }

    internal static class Handles
    {
        private static object _lock = new object();
        private static ulong nextHandleId = 1;
        private static Dictionary<ulong, KernelHandle> kernelHandles = new Dictionary<ulong, KernelHandle>();

        public static ulong Create(ulong PID, HandleType type)
        {
            lock (_lock)
            {
                var id = nextHandleId++;
                var handle = new KernelHandle(id, PID, type);
                kernelHandles.Add(id, handle);
                return handle.Id;
            }
        }

        public static ulong Create(ulong PID, ulong additionalPID, HandleType type)
        {
            lock (_lock)
            {
                var id = nextHandleId++;
                var handle = new KernelHandle(id, PID, additionalPID, type);
                kernelHandles.Add(id, handle);
                return handle.Id;
            }
        }

        public static bool Destroy(ulong PID, ulong handleId)
        {
            lock (_lock)
            {
                if (kernelHandles.TryGetValue(handleId, out var handle))
                {
                    if (handle.OwningPIDs.Contains(PID))
                    {
                        kernelHandles.Remove(handleId);
                    }
                }
                return false;
            }
        }

        public static KernelHandle GetChannelHandleForSignal(ulong handleId, ulong senderPID)
        {
            lock (_lock)
            {
                if (kernelHandles.TryGetValue(handleId, out var handle))
                {
                    if (handle.OwningPIDs.Contains(senderPID) && handle.Type == HandleType.Channel)
                    {
                        return handle;
                    }
                }
                return null;
            }
        }

        public static void CleanupAfterProcess(ulong PID)
        {
            lock (_lock)
            {
                foreach (var handle in kernelHandles.Values)
                {
                    if (handle.OwningPIDs.Contains(PID)) handle.OwningPIDs.Remove(PID);
                }
                kernelHandles = kernelHandles.Where(h => h.Value.OwningPIDs.Count > 0).ToDictionary(h => h.Key, h => h.Value);
            }
        }
    }
}
