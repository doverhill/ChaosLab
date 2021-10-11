using System;

namespace Core
{
    public class Handle
    {
        private Guid globalId;
        private ulong localId;

        public Action OnConnect { get; set; }

        public override string ToString()
        {
            if (localId != 0)
                return "L:" + localId;
            else
                return "G:" + globalId;
        }
    }
}
