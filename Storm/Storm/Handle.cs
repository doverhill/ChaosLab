using System.Collections.Generic;
using System.Linq;

namespace Storm {
    internal class Handle {
        public enum HandleType {
            Service,
            Channel,
            ServiceSubscribe,
            Timer,
            Process
        }

        public ulong Id;
        public HashSet<ulong> OwningProcessIds;
        public HandleType Type;

        public Handle(ulong handleId, ulong owningProcessId, HandleType resource) {
            Id = handleId;
            OwningProcessIds = new HashSet<ulong> { owningProcessId };
            Type = resource;
        }

        public Handle(ulong handleId, ulong owningProcessId, ulong additionalProcessId, HandleType resource) {
            Id = handleId;
            OwningProcessIds = new HashSet<ulong> { owningProcessId, additionalProcessId };
            Type = resource;
        }

        public void Close(ulong closingProcessId) {
            switch (Type) {
                case HandleType.Service:
                    ServiceCollection.Remove(Id);
                    break;

                case HandleType.Channel:
                    var otherProcessID = GetOtherProcessId(closingProcessId);
                    var otherProcess = Process.FindProcess(otherProcessID);
                    otherProcess.PostChannelClosedEvent(Id);
                    break;

                case HandleType.ServiceSubscribe:
                    ServiceCollection.RemoveSubscription(Id);
                    break;

                case HandleType.Timer:
                    TimerCollection.Remove(Id);
                    break;
            }
        }

        public ulong GetOtherProcessId(ulong processId) {
            if (OwningProcessIds.Contains(processId) && OwningProcessIds.Count == 2) {
                return OwningProcessIds.First(p => processId != p);
            }
            ASSERT.NotReached();
            return ulong.MaxValue;
        }
    }
}
