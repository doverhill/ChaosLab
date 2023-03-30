using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;

namespace Storm {
    internal class Process {
        private static object _globalLock = new object();
        private static Dictionary<ulong, Process> _processes = new Dictionary<ulong, Process>();

        public class Thread {
            public enum ThreadState {
                Running,
                WaitEvent
            }

            public ulong ThreadId;
            public ThreadState State;

            internal Thread(ulong threadId) {
                ThreadId = threadId;
            }
        }

        public ulong ProcessId;
        public string TrustChain;
        public List<Capability> Capabilities;
        public List<Capability> Grantables;
        public Dictionary<ulong, Thread> Threads;

        private HashSet<ulong> _signalledChannels = new();
        private BlockingCollection<Event> _eventQueue = new();

        public static ErrorOr<Process> CreateProcess(ulong processId, Process parent, string name, List<string> capabilityStrings, List<string> grantableStrings) {
            var capabilities = new List<Capability>();
            foreach (var capabilityString in capabilityStrings) {
                var result = Capability.Parse(capabilityString);
                if (result.IsError) return ErrorOr<Process>.Error(result.ErrorCode);
                capabilities.Add(result.Value);
            }

            var grantables = new List<Capability>();
            foreach (var grantableString in grantableStrings) {
                var result = Capability.Parse(grantableString);
                if (result.IsError) return ErrorOr<Process>.Error(result.ErrorCode);
                grantables.Add(result.Value);
            }

            // check that the parent has the right to give the new process these capabilities and grantables and also that grantables is a subset of capabilities
            if (!Capability.IsSubset(parent.Grantables, capabilities) || !Capability.IsSubset(parent.Grantables, grantables) || !Capability.IsSubset(capabilities, grantables))
                return ErrorOr<Process>.Error(ErrorCode.PermissionDenied);

            var trustChain = $"{parent.TrustChain}.{name}";

            var process = new Process {
                Capabilities = capabilities,
                Grantables = grantables,
                ProcessId = processId,
                Threads = new(),
                TrustChain = trustChain
            };

            lock (_globalLock) {
                _processes.Add(processId, process);
            }

            return ErrorOr<Process>.Ok(process);
        }

        public static ErrorOr<(Process Process, Thread Thread)> GetProcess(ulong processId, ulong threadId) {
            lock (_globalLock) {
                if (!_processes.TryGetValue(processId, out var process)) {
                    return ErrorOr<(Process, Thread)>.Error(ErrorCode.NotFound);
                }

                Thread thread = null;
                if (!process.Threads.TryGetValue(threadId, out thread)) {
                    thread = new Thread(threadId);
                    process.Threads.Add(threadId, thread);
                }

                return ErrorOr<(Process, Thread)>.Ok((process, thread));
            }
        }

        public static Process FindProcess(ulong processId) {
            if (_processes.TryGetValue(processId, out var process)) return process;
            return null;
        }

        public static bool RemoveThread(Process process, Thread thread) {
            lock (_globalLock) {
                process.Threads.Remove(thread.ThreadId);
                if (process.Threads.Count == 0) {
                    _processes.Remove(process.ProcessId);
                    return true;
                }
                return false;
            }
        }

        //public static void FireEvent(Event stormEvent) {
        //    if (_processes.TryGetValue(stormEvent.TargetProcessId, out var process)) {
        //        process.QueueEvent(stormEvent);
        //    }
        //}

        public void SetChannelSignalled(ulong channelHandleId) {
            _signalledChannels.Add(channelHandleId);
        }

        public void QueueEvent(Event stormEvent) {
            //ASSERT.That(stormEvent.TargetProcessId == ProcessId);
            _eventQueue.Add(stormEvent);
        }

        public void PostServiceAvailableEvent(ulong serviceSubscribeHandleId, ulong channelHandleId) {
            var stormEvent = new Event(Event.EventType.ServiceAvailable, serviceSubscribeHandleId, channelHandleId);
            QueueEvent(stormEvent);
        }

        public void PostProcessFlagsEvent() {
            var stormEvent = new Event(Event.EventType.ProcessFlags, 0, 0);
            QueueEvent(stormEvent);
        }

        public bool WaitEvent(ulong? targetHandleId, HandleAction? action, out Event stormEvent, int timeout) {
            // FIXME understand event type ProcessFlags and check process flags and create events from them
            if (_eventQueue.TryTake(out stormEvent, timeout)) {
                ASSERT.That(stormEvent.TargetProcessId == ProcessId);
                if (stormEvent.)
            }
            else {
                stormEvent = default;
                return false;
            }
        }

        internal bool HasStormCapability(string operation, string resourceName) {
            return Capabilities.Any(c => c.Namespace == "Storm" && c.Operation == operation && c.ResourceType == Capability.Type.Name && c.ResourceName == resourceName);
        }

        internal bool HasStormCapability(string operation) {
            return Capabilities.Any(c => c.Namespace == "Storm" && c.Operation == operation && c.ResourceType == Capability.Type.None);
        }

        internal bool HasStormCapability(string operation, ulong value) {
            return Capabilities.Any(c => c.Namespace == "Storm" && c.Operation == operation && c.ResourceType == Capability.Type.Numeric && c.NumericValue == value);
        }

        internal bool HasStormCapability(string operation, ulong rangeStart, ulong rangeEnd) {
            return Capabilities.Any(c => c.Namespace == "Storm" && c.Operation == operation && c.ResourceType == Capability.Type.NumericRange && c.NumericValue <= rangeStart && c.NumericEndValue >= rangeEnd);
        }
    }
}
