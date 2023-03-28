namespace Storm {
    public enum HandleAction
    {
        None = 0,
        ServiceConnected = 1,
        ChannelSignalled = 2,
        ChannelDestroyed = 3
    }

    public class Handle
    {
        public static readonly ulong None = 0;

        public ulong Id;

        public Handle(ulong id)
        {
            Id = id;
        }

        public Action OnConnect { get; set; }

        public override string ToString()
        {
            return "[HANDLE: id=" + Id + "]";
        }
    }
}
