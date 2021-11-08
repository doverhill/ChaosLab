using Core;
using System.Collections.Concurrent;
using System.Net.Sockets;

namespace Storm
{
    internal class Event
    {
        public ulong TargetPID;
        public Error Error;
        public ulong TargetHandle;
        public ulong ChannelHandle;
        public HandleAction Action;
        public ulong Message;

        public Event(ulong targetPID, Error error, ulong targetHandle, ulong channelHandle, HandleAction action, ulong message)
        {
            TargetPID = targetPID;
            Error = error;
            TargetHandle = targetHandle;
            ChannelHandle = channelHandle;
            Action = action;
            Message = message;
        }
    }

    internal class Events
    {
        private static object _lock = new object();
        private static Dictionary<ulong, BlockingCollection<Event>> processEventQueues = new Dictionary<ulong, BlockingCollection<Event>>();

        public static void Fire(Event e)
        {
            Output.WriteLineKernel(SyscallProcessEmitType.Debug, null, "Firing event: targetPID=" + e.TargetPID + ", error=" + e.Error.ToString() + ", targetHandle=" + e.TargetHandle + ", channelHandle=" + e.ChannelHandle + ", action=" + e.Action.ToString() + ", message=" + e.Message);
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

        private static bool SocketConnected(Socket s)
        {
            bool part1 = s.Poll(1000, SelectMode.SelectRead);
            bool part2 = (s.Available == 0);
            if (part1 && part2)
                return false;
            else
                return true;
        }

        private static bool EventMatches(Event e, ulong? handleId, HandleAction? action, ulong? message)
        {
            if (handleId.HasValue && e.TargetHandle != handleId.Value) return false;
            if (action.HasValue && e.Action != action.Value) return false;
            if (message.HasValue && e.Message != message.Value) return false;
            return true;
        }

        public static Event Wait(Socket socket, ulong PID, ulong? handleId, HandleAction? action, ulong? message, int timeoutMilliseconds)
        {
            BlockingCollection<Event> eventQueue = null;

            lock (_lock)
            {
                if (!processEventQueues.TryGetValue(PID, out eventQueue))
                {
                    eventQueue = new BlockingCollection<Event>();
                    processEventQueues.Add(PID, eventQueue);
                }
            }

            int totalTime = 0;
            var eventsToPutBack = new List<Event>();
            while (timeoutMilliseconds == -1 || totalTime < timeoutMilliseconds)
            {
                if (eventQueue.TryTake(out var e, 100))
                {
                    Output.WriteLineKernel(SyscallProcessEmitType.Debug, null, "Received event: targetPID=" + e.TargetPID + ", error=" + e.Error.ToString() + ", targetHandle=" + e.TargetHandle + ", channelHandle=" + e.ChannelHandle + ", action=" + e.Action.ToString() + ", message=" + e.Message);
                    if (EventMatches(e, handleId, action, message))
                    {
                        foreach (var putback in eventsToPutBack)
                        {
                            eventQueue.Add(putback);
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
            return new Event(PID, Error.Timeout, Handle.None, Handle.None, HandleAction.None, 0);
        }

        public static void CleanupAfterProcess(ulong PID)
        {
            lock (_lock)
            {
                processEventQueues.Remove(PID);
            }
        }
    }
}
