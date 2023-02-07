using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Storm
{
    internal class Process
    {
        public class Thread
        {
            public ulong ThreadId;

            internal Thread(ulong threadId)
            {
                ThreadId = threadId;
            }
        }

        public ulong ProcessId;
        public string Name;
        public Dictionary<ulong, Thread> Threads;

        private BlockingCollection<Event> _eventQueue = new BlockingCollection<Event>();
        private object _processLock = new object();

        private Process(ulong processId, Thread thread)
        {
            ProcessId = processId;
            Name = "?";
            Threads = new Dictionary<ulong, Thread>
            {
                { thread.ThreadId, thread }
            };
        }

        private static object _globalLock = new object();
        private static Dictionary<ulong, Process> _processes = new Dictionary<ulong, Process>();

        public static (Process Process, Thread Thread) GetProcess(ulong processId, ulong threadId)
        {
            lock (_globalLock)
            {
                var thread = new Thread(threadId);
                if (!_processes.TryGetValue(processId, out var process))
                {
                    process = new Process(processId, thread);
                    _processes.Add(processId, process);
                }
                else
                {
                    process.Threads.Add(threadId, thread);
                }

                return (process, thread);
            }
        }

        public static Process FindProcess(ulong processId)
        {
            if (_processes.TryGetValue(processId, out var process)) return process;
            return null;
        }

        public static bool Cleanup(Process process, Process.Thread thread)
        {
            lock (_globalLock)
            {
                process.Threads.Remove(thread.ThreadId);
                if (process.Threads.Count == 0)
                {
                    _processes.Remove(process.ProcessId);
                    return true;
                }
                return false;
            }
        }

        public static void FireEvent(Event stormEvent)
        {
            if (_processes.TryGetValue(stormEvent.TargetPID, out var process))
            {
                process.QueueEvent(stormEvent);
            }
        }

        public void QueueEvent(Event stormEvent)
        {
            _eventQueue.Add(stormEvent);
        }

        public bool WaitEvent(out Event stormEvent, int timeout)
        {
            return _eventQueue.TryTake(out stormEvent, timeout);
        }
    }
}
