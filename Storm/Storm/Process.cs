using System.Collections.Concurrent;

namespace Storm {
    internal class Process {
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
        public string Name;
        public string TrustChain;
        public Dictionary<ulong, Thread> Threads;

        private BlockingCollection<Event> _eventQueue = new BlockingCollection<Event>();

        private Process(ulong processId, Thread thread, string name) {
            ProcessId = processId;
            Name = name;
            Threads = new Dictionary<ulong, Thread> {
                { thread.ThreadId, thread }
            };
        }

        private static object _globalLock = new object();
        private static Dictionary<ulong, Process> _processes = new Dictionary<ulong, Process>();

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
