using Core;
using System.Collections.Concurrent;

namespace Storm
{
    internal class Event
    {
        public ulong TargetPID;
        public Error Error;
        public ulong TargetHandle;
        public ulong ArgumentHandle;
        public HandleAction Action;
        public ulong Parameter;

        public Event(ulong targetPID, Error error, ulong targetHandle, ulong argumentHandle, HandleAction action, ulong parameter)
        {
            TargetPID = targetPID;
            Error = error;
            TargetHandle = targetHandle;
            ArgumentHandle = argumentHandle;
            Action = action;
            Parameter = parameter;
        }
    }

    internal class Events
    {
        private static object _lock = new object();
        private static Dictionary<ulong, BlockingCollection<Event>> processEventQueues = new Dictionary<ulong, BlockingCollection<Event>>();

        public static void Fire(Event e)
        {
            Output.WriteLine(SyscallProcessEmitType.Debug, null, "Firing event: targetPID=" + e.TargetPID + ", error=" + e.Error.ToString() + ", targetHandle=" + e.TargetHandle + ", argumentHandle=" + e.ArgumentHandle + ", action=" + e.Action.ToString() + ", parameter=" + e.Parameter);
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
                Output.WriteLine(SyscallProcessEmitType.Debug, null, "Received event: targetPID=" + e.TargetPID + ", error=" + e.Error.ToString() + ", targetHandle=" + e.TargetHandle + ", argumentHandle=" + e.ArgumentHandle + ", action=" + e.Action.ToString() + ", parameter=" + e.Parameter);
                return e;
            }
            return new Event(pid, Error.Timeout, Handle.None, Handle.None, HandleAction.None, 0);
        }
    }
}
