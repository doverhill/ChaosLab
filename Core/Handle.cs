using System;

namespace Core
{
    public enum HandleAction
    {
        Unknown = 0,
        Connect = 1,
        Open = 2,
        Close = 3,
        Read = 4,
        Write = 5
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
            return "H" + Id;
        }
    }
}
