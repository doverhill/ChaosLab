using System;

namespace Core
{
    public enum HandleAction
    {
        Connect = 1,
        Open = 2,
        Close = 3,
        Read = 4,
        Write = 5
    }

    public class Handle
    {
        //private bool isGlobal;
        //private ulong localId;
        //private Guid? globalId;

        //public Handle(bool isGlobal, ulong localId, Guid? globalId)
        //{
        //    this.isGlobal = isGlobal;
        //    this.localId = localId;
        //    this.globalId = globalId;
        //}

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

        //public override string ToString()
        //{
        //    if (isGlobal)
        //        return "G:" + globalId;
        //    else
        //        return "L:" + localId;
        //}
    }
}
