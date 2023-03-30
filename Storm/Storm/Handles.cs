using System.Collections.Generic;
using System.Linq;

namespace Storm {
    internal class Handle {
        public enum Type {
            Service,
            Channel,
            ServiceSubscribe,
            Timer,
            Process
        }

        public ulong Id;
        public HashSet<ulong> OwningProcessIds;
        public Type Resource;

        public Handle(ulong handleId, ulong owningProcessId, Type resource) {
            Id = handleId;
            OwningProcessIds = new HashSet<ulong> { owningProcessId };
            Resource = resource;
        }

        public Handle(ulong handleId, ulong owningProcessId, ulong additionalProcessId, Type resource) {
            Id = handleId;
            OwningProcessIds = new HashSet<ulong> { owningProcessId, additionalProcessId };
            Resource = resource;
        }

        public void Close(ulong closingProcessId) {
            switch (Resource) {
                case Type.Service:
                    // FIXME close all handles connected to this service
                    break;

                case Type.Channel:
                    // FIXME close other end
                    break;

                case Type.ServiceSubscribe:
                    // FIXME
                    break;

                case Type.Timer:
                    //FIXME
                    break;
            }
            //var otherProcessId = GetOtherProcessId(closingProcessId);
            //if (otherProcessId.HasValue) {
            //    var process = Process.FindProcess(otherProcessId.Value);
            //    var stormEvent = new Event(otherProcessId.Value, Error.None, Id, Handle.None, HandleAction.ChannelDestroyed);
            //    Process.FireEvent(stormEvent);
            //}
        }

        public ulong GetOtherProcessId(ulong processId) {
            if (OwningProcessIds.Contains(processId) && OwningProcessIds.Count == 2) {
                return OwningProcessIds.First(p => processId != p);
            }
            ASSERT.NotReached();
            return ulong.MaxValue;
        }
    }

    internal static class Handles {
        private static object _lock = new object();
        private static ulong _nextHandleId = 1;
        private static Dictionary<ulong, Handle> _handles = new();

        public static ulong Create(ulong forProcessId, Handle.Type resource) {
            lock (_lock) {
                var id = _nextHandleId++;
                var handle = new Handle(id, forProcessId, resource);
                _handles.Add(id, handle);
                return handle.Id;
            }
        }

        public static ulong Create(ulong forProcessId, ulong additionalProcessId, Handle.Type resource) {
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
                        handle.Close();
                        _handles.Remove(handleId);
                        return;
                    }
                }
            }
            ASSERT.NotReached();
        }

        public static Optional<Handle> GetHandle(ulong processId, ulong handleId, Handle.Type resource) {
            lock (_lock) {
                if (_handles.TryGetValue(handleId, out var handle)) {
                    if (handle.OwningProcessIds.Contains(processId) && handle.Resource == resource) {
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
