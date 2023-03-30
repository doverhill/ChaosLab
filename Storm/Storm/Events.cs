using System;
using System.Collections.Generic;
using System.Net.Sockets;

namespace Storm {
    internal class Event
    {
        public enum EventType {
            ProcessFlags,
            ServiceAvailable,
            ChannelClosed,
            TimerFired,
        }

        //public ulong TargetProcessId;
        public EventType Type;
        //public ErrorCode Error;
        public ulong TargetHandleId;
        public ulong AdditionalHandleId;

        public Event(EventType type, ulong targetHandleId, ulong additionalHandleId) {
            Type = type;
            TargetHandleId = targetHandleId;
            AdditionalHandleId = additionalHandleId;
        }
        //public Handle ChannelHandle;
        //public HandleAction Action;

        //public Event(ulong targetPID, ErrorCode error, Handle targetHandle, Handle channelHandle, HandleAction action)
        //{
        //    TargetProcessId = targetPID;
        //    Error = error;
        //    TargetHandle = targetHandle;
        //    ChannelHandle = channelHandle;
        //    Action = action;
        //}


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

        public static Optional<Event> Wait(Socket socket, Process process, Process.Thread thread, ulong? targetHandleId, HandleAction? action, int timeoutMilliseconds)
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
                if (process.WaitEvent(targetHandleId, action, out var stormEvent, 500))
                {
                    Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "Received event: targetProcessId=" + process.ProcessId + ", targetHandleId=" + stormEvent.TargetHandleId + ", additionalHandleId=" + stormEvent.AdditionalHandleId + ", type=" + stormEvent.Type.ToString());
                    return Optional<Event>.WithValue(stormEvent);
                }
                if (!SocketConnected(socket)) throw new Exception("Socket was closed, killing application");
                totalTime += 500;
            }
            return Optional<Event>.None();
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
