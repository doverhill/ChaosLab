using System.Collections.Concurrent;

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
        public List<string> Capabilities;
        public List<string> Grantables;
        public Dictionary<ulong, Thread> Threads;

        private BlockingCollection<Event> _eventQueue = new BlockingCollection<Event>();

        public static Process TryCreateProcess(ulong processId, Process parent, string name, List<string> capabilites, List<string> grantables) {
            // first, ensure that all capabiltites and grantables are correctly formatted
            foreach (var )

                // ensure grantables are a subset of capabilities


                ProcessId = processId;
            Name = name;
            Threads = new Dictionary<ulong, Thread> {
                { thread.ThreadId, thread }
            };
        }

        public static (Process Process, Thread Thread) GetProcess(ulong processId, ulong threadId, string name) {
            lock (_globalLock) {
                var thread = new Thread(threadId);
                if (!_processes.TryGetValue(processId, out var process)) {
                    process = new Process(processId, thread, name);
                    _processes.Add(processId, process);
                }
                else {
                    process.Threads.Add(threadId, thread);
                }

                return (process, thread);
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

        public static void FireEvent(Event stormEvent) {
            if (_processes.TryGetValue(stormEvent.TargetProcessId, out var process)) {
                process.QueueEvent(stormEvent);
            }
        }

        public void QueueEvent(Event stormEvent) {
            ASSERT.That(stormEvent.TargetProcessId == ProcessId);
            _eventQueue.Add(stormEvent);
        }

        public bool WaitEvent(out Event stormEvent, int timeout) {
            return _eventQueue.TryTake(out stormEvent, timeout);
        }
    }
}
