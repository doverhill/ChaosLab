namespace Core
{
    public static class Service
    {
        public static ErrorOr<Handle> Create(ServiceDescription description)
        {
            return Syscalls.ServiceCreate(description);

            register handle globally so that event loop can use it and call On* methods on it when there is an event for the handle id
        }

        public static Optional<Error> Destroy(Handle serviceHandle)
        {
            return new Optional<Error>(Error.NotImplemented);
        }
    }
}
