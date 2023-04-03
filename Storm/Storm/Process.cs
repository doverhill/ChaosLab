using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Net.Sockets;
using System.Threading;

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

        private HashSet<ulong> _signalledChannelIds = new();
        private BlockingCollection<Event> _eventQueue = new();
        private CancellationTokenSource _eventQueueCancellation = new();

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
            _signalledChannelIds.Add(channelHandleId);
            _eventQueueCancellation.Cancel();
        }

        public void QueueEvent(Event stormEvent) {
            //ASSERT.That(stormEvent.TargetProcessId == ProcessId);
            _eventQueue.Add(stormEvent);
        }

        public void PostChannelClosedEvent(ulong channelHandleId) {
            var stormEvent = new Event(Event.EventType.ChannelDestroyed, channelHandleId, 0);
            QueueEvent(stormEvent);
        }

        public void PostServiceAvailableEvent(ulong serviceSubscribeHandleId, ulong channelHandleId) {
            var stormEvent = new Event(Event.EventType.ServiceAvailable, serviceSubscribeHandleId, channelHandleId);
            QueueEvent(stormEvent);
        }

        //public void PostProcessFlagsEvent() {
        //    var stormEvent = new Event(Event.EventType.ProcessFlags, 0, 0);
        //    QueueEvent(stormEvent);
        //}

        private static bool EventMatches(Event stormEvent, ulong? targetHandleId, Event.EventType? eventType) {
            if (targetHandleId.HasValue && stormEvent.TargetHandleId != targetHandleId.Value) return false;
            if (eventType.HasValue && stormEvent.Type != eventType.Value) return false;
            return true;
        }

        private static bool IsSocketConnected(Socket socket) {
            bool part1 = socket.Poll(1000, SelectMode.SelectRead);
            bool part2 = socket.Available == 0;
            if (part1 && part2)
                return false;
            else
                return true;
        }

        public bool WaitEvent(Socket socket, ulong? targetHandleId, Event.EventType? eventType, out Event stormEvent, int timeoutMilliseconds) {


            int totalTime = 0;
            //var eventsToPutBack = new List<Event>();
            while (timeoutMilliseconds == -1 || totalTime < timeoutMilliseconds) {
                if (process.WaitEvent(targetHandleId, action, out var stormEvent, 500)) {
                    Output.WriteLineKernel(ProcessEmitType.Debug, process, thread, "Received event: targetProcessId=" + process.ProcessId + ", targetHandleId=" + stormEvent.TargetHandleId + ", additionalHandleId=" + stormEvent.AdditionalHandleId + ", type=" + stormEvent.Type.ToString());
                    return Optional<Event>.WithValue(stormEvent);
                }
                if (!IsSocketConnected(socket)) throw new Exception("Socket was closed, killing application");
                totalTime += 500;
            }
            return Optional<Event>.None();



            // FIXME figure this out!!!?!?!

            // FIXME understand event type ProcessFlags and check process flags and create events from them
            if (_eventQueue.TryTake(out stormEvent, timeout)) {
                if (stormEvent.Type == Event.EventType.ProcessFlags) {
                    if (_signalledChannelIds.Count > 0) {
                        var signalledChannelId = _signalledChannelIds.First();
                        _signalledChannelIds.Remove(signalledChannelId);
                        stormEvent = new Event(Event.EventType.ChannelSignalled, signalledChannelId, 0);
                        return true;
                    }
                }
                else {
                    return true;
                }
            }
            else {
                stormEvent = default;
                return false;
            }
        }

        internal bool HasStormCapability(string operation, string resourceName) {
            return Capabilities.Any(c => (c.Namespace == "*" || c.Namespace == "Storm") && (c.Operation == "*" || c.Operation == operation) && (c.Type == Capability.CapabilityType.Any || c.Type == Capability.CapabilityType.Name && c.ResourceName == resourceName));
        }

        internal bool HasStormCapability(string operation) {
            return Capabilities.Any(c => (c.Namespace == "*" || c.Namespace == "Storm") && (c.Operation == "*" || c.Operation == operation) && (c.Type == Capability.CapabilityType.Any || c.Type == Capability.CapabilityType.None));
        }

        internal bool HasStormCapability(string operation, ulong value) {
            return Capabilities.Any(c => (c.Namespace == "*" || c.Namespace == "Storm") && (c.Operation == "*" || c.Operation == operation) && (c.Type == Capability.CapabilityType.Any || c.Type == Capability.CapabilityType.Numeric && c.NumericValue == value));
        }

        internal bool HasStormCapability(string operation, ulong rangeStart, ulong rangeEnd) {
            return Capabilities.Any(c => (c.Namespace == "*" || c.Namespace == "Storm") && (c.Operation == "*" || c.Operation == operation) && (c.Type == Capability.CapabilityType.Any || c.Type == Capability.CapabilityType.NumericRange && c.NumericValue <= rangeStart && c.NumericEndValue >= rangeEnd));
        }
    }
}
