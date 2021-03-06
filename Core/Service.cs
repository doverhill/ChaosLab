using Uuids;

namespace Core
{
    public static class Service
    {
        public static ErrorOr<Handle> Create(string protocolName, string vendorName, string deviceName, Uuid deviceId)
        {
            var handle = Syscalls.ServiceCreate(protocolName, vendorName, deviceName, deviceId);
            if (!handle.IsError()) Process.RegisterHandle(handle.Value());
            return handle;
        }

        public static ErrorOr<Channel> Connect(string protocolName, string vendorName, string deviceName, Uuid? deviceId)
        {
            var result = Syscalls.ServiceConnect(protocolName, vendorName, deviceName, deviceId);
            if (result.IsError()) return new ErrorOr<Channel>(result.Error());
            return new ErrorOr<Channel>(new Channel(result.Value()));
        }

        public static Optional<Error> Destroy(Handle serviceHandle)
        {
            return new Optional<Error>(Error.NotImplemented);
        }
    }
}
