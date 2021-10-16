namespace Core
{
    public static class Service
    {
        public static ErrorOr<Handle> Create(ServiceDescription description)
        {
            var handle = Syscalls.ServiceCreate(description);
            if (!handle.IsError()) Process.RegisterHandle(handle.Value());
            return handle;
        }

        public static Optional<Error> Destroy(Handle serviceHandle)
        {
            return new Optional<Error>(Error.NotImplemented);
        }
    }
}
