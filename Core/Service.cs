namespace Core
{
    public static class Service
    {
        public static ErrorOr<Handle> Create(ServiceDescription description)
        {
            return new ErrorOr<Handle>(Error.NotImplemented);
        }

        public static Optional<Error> Destroy(Handle serviceHandle)
        {
            return new Optional<Error>(Error.NotImplemented);
        }
    }
}
