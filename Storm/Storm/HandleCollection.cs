using System.Collections.Generic;

namespace Storm {
    internal static class HandleCollection {
        private static object _lock = new object();
        private static ulong _nextHandleId = 1;
        private static Dictionary<ulong, Handle> _handles = new();

        public static ulong Create(ulong forProcessId, Handle.HandleType resource) {
            lock (_lock) {
                var id = _nextHandleId++;
                var handle = new Handle(id, forProcessId, resource);
                _handles.Add(id, handle);
                return handle.Id;
            }
        }

        public static ulong Create(ulong forProcessId, ulong additionalProcessId, Handle.HandleType resource) {
            lock (_lock) {
                var id = _nextHandleId++;
                var handle = new Handle(id, forProcessId, additionalProcessId, resource);
                _handles.Add(id, handle);
                return handle.Id;
            }
        }

        public static void Close(ulong processId, ulong handleId) {
            lock (_lock) {
                if (_handles.TryGetValue(handleId, out var handle)) {
                    if (handle.OwningProcessIds.Contains(processId)) {
                        handle.Close(processId);
                        _handles.Remove(handleId);
                        return;
                    }
                }
            }
            ASSERT.NotReached();
        }

        public static Optional<Handle> GetHandle(ulong processId, ulong handleId, Handle.HandleType resource) {
            lock (_lock) {
                if (_handles.TryGetValue(handleId, out var handle)) {
                    if (handle.OwningProcessIds.Contains(processId) && handle.Type == resource) {
                        return Optional<Handle>.WithValue(handle);
                    }
                }
            }
            return Optional<Handle>.None();
        }

        public static void CleanupAfterProcess(Process process) {
            lock (_lock) {
                var closeHandleIds = new List<ulong>();
                foreach (var handle in _handles.Values) {
                    if (handle.OwningProcessIds.Contains(process.ProcessId)) closeHandleIds.Add(handle.Id);
                }
                foreach (var handleId in closeHandleIds) {
                    Close(process.ProcessId, handleId);
                }
            }
        }
    }
}
