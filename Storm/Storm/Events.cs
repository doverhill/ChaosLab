using System.Net.Sockets;

namespace Storm {
    internal class Event
    {
        public ulong TargetProcessId;
        public Error Error;
        public ulong TargetHandle;
        public ulong ChannelHandle;
        public HandleAction Action;

        public Event(ulong targetPID, Error error, ulong targetHandle, ulong channelHandle, HandleAction action)
        {
            TargetProcessId = targetPID;
            Error = error;
            TargetHandle = targetHandle;
            ChannelHandle = channelHandle;
            Action = action;
        }
    }

    internal class Events
    {
        //private static object _lock = new object();
        //private static Dictionary<ulong, BlockingCollection<Event>> processEventQueues = new Dictionary<ulong, BlockingCollection<Event>>();

        //public static void Fire(Process process, Event e)
        //{
        //    Output.WriteLineKernel(SyscallProcessEmitType.Debug, null, "Firing event: targetPID=" + e.TargetPID + ", error=" + e.Error.ToString() + ", targetHandle=" + e.TargetHandle + ", channelHandle=" + e.ChannelHandle + ", action=" + e.Action.ToString());
        //    lock (_lock)
        //    {
        //        if (!processEventQueues.TryGetValue(e.TargetPID, out var eventQueue))
        //        {
        //            eventQueue = new BlockingCollection<Event>();
        //            processEventQueues.Add(e.TargetPID, eventQueue);
        //        }

        //        eventQueue.Add(e);
        //    }
        //}

        private static bool SocketConnected(Socket s)
        {
            bool part1 = s.Poll(1000, SelectMode.SelectRead);
            bool part2 = (s.Available == 0);
            if (part1 && part2)
                return false;
            else
                return true;
        }

        private static bool EventMatches(Event e, ulong? handleId, HandleAction? action)
        {
            if (handleId.HasValue && e.TargetHandle != handleId.Value) return false;
            if (action.HasValue && e.Action != action.Value) return false;
            return true;
        }

        public static Event Wait(Socket socket, Process process, ulong? handleId, HandleAction? action, int timeoutMilliseconds)
        {
            //BlockingCollection<Event> eventQueue = null;

            //lock (_lock)
            //{
            //    if (!processEventQueues.TryGetValue(PID, out eventQueue))
            //    {
            //        eventQueue = new BlockingCollection<Event>();
            //        processEventQueues.Add(PID, eventQueue);
            //    }
            //}

            int totalTime = 0;
            var eventsToPutBack = new List<Event>();
            while (timeoutMilliseconds == -1 || totalTime < timeoutMilliseconds)
            {
                if (process.WaitEvent(out var e, 100))
                {
                    Output.WriteLineKernel(SyscallProcessEmitType.Debug, null, null, "Received event: targetPID=" + e.TargetProcessId + ", error=" + e.Error.ToString() + ", targetHandle=" + e.TargetHandle + ", channelHandle=" + e.ChannelHandle + ", action=" + e.Action.ToString());
                    if (EventMatches(e, handleId, action))
                    {
                        foreach (var putback in eventsToPutBack)
                        {
                            process.QueueEvent(putback);
                        }
                        return e;
                    }
                    else
                    {
                        eventsToPutBack.Add(e);
                    }
                }
                if (!SocketConnected(socket)) throw new Exception("Socket was closed, killing application");
                totalTime += 100;
            }
            return new Event(process.ProcessId, Error.Timeout, Handle.None, Handle.None, HandleAction.None);
        }

        //public static void CleanupAfterProcess(ulong PID)
        //{
        //    lock (_lock)
        //    {
        //        processEventQueues.Remove(PID);
        //    }
        //}
    }
}
