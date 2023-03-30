namespace Storm {
    internal class Event {
        public enum EventType {
            None = 0,
            ServiceConnected = 100,
            ServiceAvailable = 101,

            ChannelSignalled = 200,
            ChannelDestroyed = 201,

            TimerFired = 300,

            ProcessExited = 400,
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
}
