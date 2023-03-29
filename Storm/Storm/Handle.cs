namespace Storm {
    public enum HandleAction
    {
        None = 0,
        ServiceConnected = 100,
        ServiceAvailable = 101,

        ChannelSignalled = 200,
        ChannelDestroyed = 201,

        TimerFired = 300,

        ProcessExited = 400,
    }

    we don't need two types of handles!

    public class Handle
    {
        public enum Type {
            Service,
            Channel,
            ServiceSubscribe,
            Timer
        }

        public ulong Id;
        public Type Resource;

        public Handle(ulong id, Type resource)
        {
            Id = id;
            Resource = resource;
        }

        public override string ToString()
        {
            return $"[HANDLE: resource={Resource}, id={Id}]";
        }
    }
}
