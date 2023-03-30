using Uuids;

namespace Storm {
    internal class Service {
        public ulong OwningProcessId;
        public ulong HandleId;
        public string Protocol;
        public string Owner;
        public Uuid DeviceId;

        public Service(ulong owningProcessId, ulong handleId, string protocol, string owner, Uuid deviceId) {
            OwningProcessId = owningProcessId;
            HandleId = handleId;
            Protocol = protocol;
            Owner = owner;
            DeviceId = deviceId;
        }
    }
}
