namespace Storm {
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

        public void Close(ulong processId)
        {
            var otherProcessId = GetOtherProcessId(processId);
            if (otherProcessId != Handle.None)
            {
                var process = Process.FindProcess(otherProcessId);
                var stormEvent = new Event(otherProcessId, Error.None, Id, Handle.None, HandleAction.ChannelDestroyed);
                Process.FireEvent(stormEvent);
            }
        }

        public ulong GetOtherProcessId(ulong processId)
        {
            if (OwningPIDs.Contains(processId) && OwningPIDs.Count == 2)
            {
                var otherProcessId = OwningPIDs.Where(p => processId != p).First();
                return otherProcessId;
            }
            return Handle.None;
        }
    }

    internal static class Handles
    {
        private static object _lock = new object();
        private static ulong _nextHandleId = 1;
        private static Dictionary<ulong, KernelHandle> _kernelHandles = new Dictionary<ulong, KernelHandle>();

        public static ulong Create(ulong ProcessId, HandleType type)
        {
            lock (_lock)
            {
                var id = _nextHandleId++;
                var handle = new KernelHandle(id, PID, type);
                _kernelHandles.Add(id, handle);
                return handle.Id;
            }
        }

        public static ulong Create(ulong PID, ulong additionalPID, HandleType type)
        {
            lock (_lock)
            {
                var id = _nextHandleId++;
                var handle = new KernelHandle(id, PID, additionalPID, type);
                _kernelHandles.Add(id, handle);
                return handle.Id;
            }
        }

        public static bool Destroy(ulong PID, ulong handleId)
        {
            lock (_lock)
            {
                if (_kernelHandles.TryGetValue(handleId, out var handle))
                {
                    if (handle.OwningPIDs.Contains(PID))
                    {
                        _kernelHandles.Remove(handleId);
                    }
                }
                return false;
            }
        }

        public static KernelHandle GetChannelHandleForSignal(ulong handleId, ulong senderPID)
        {
            lock (_lock)
            {
                if (_kernelHandles.TryGetValue(handleId, out var handle))
                {
                    if (handle.OwningPIDs.Contains(senderPID) && handle.Type == HandleType.Channel)
                    {
                        return handle;
                    }
                }
                return null;
            }
        }

        public static void Cleanup(Process process)
        {
            lock (_lock)
            {
                foreach (var handle in _kernelHandles.Values)
                {
                    if (handle.OwningPIDs.Contains(process.ProcessId)) handle.Close(process.ProcessId);
                }
                _kernelHandles = _kernelHandles.Where(h => h.Value.OwningPIDs.Count > 0).ToDictionary(h => h.Key, h => h.Value);
            }
        }
    }
}
