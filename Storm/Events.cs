using Core;
using System.Collections.Concurrent;

namespace Storm
{
    internal class Event
    {
        public ulong TargetPID;
        public Error Error;
        public ulong Handle;
        public HandleAction Action;

        public Event(ulong targetPID, Error error, ulong handle, HandleAction action)
        {
            TargetPID = targetPID;
            Error = error;
            Handle = handle;
            Action = action;
        }
    }

    internal class Events
    {
        private static object _lock = new object();
        private static Dictionary<ulong, BlockingCollection<Event>> processEventQueues = new Dictionary<ulong, BlockingCollection<Event>>();

        public static void Fire(Event e)
        {
            lock (_lock)
            {
                if (!processEventQueues.TryGetValue(e.TargetPID, out var eventQueue))
                {
                    eventQueue = new BlockingCollection<Event>();
                    processEventQueues.Add(e.TargetPID, eventQueue);
                }

                eventQueue.Add(e);
            }
        }

        public static Event Wait(ulong pid, int timeoutMilliseconds)
        {
            BlockingCollection<Event> eventQueue = null;

            lock (_lock)
            {
                if (!processEventQueues.TryGetValue(pid, out eventQueue))
                {
                    eventQueue = new BlockingCollection<Event>();
                    processEventQueues.Add(pid, eventQueue);
                }
            }

            // there was nothing in the queue, sleep waiting
            if (eventQueue.TryTake(out var e, timeoutMilliseconds))
            {
                return e;
            }
            return new Event(pid, Error.Timeout, 0, HandleAction.Unknown);
        }
    }
}
