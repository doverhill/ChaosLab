namespace Core
{
    public static class Process
    {
        public static Optional<Error> Run()
        {
            return new Optional<Error>(Error.NotImplemented);
        }

        public static void End()
        {
            EmitDebug("Application end");
            //InvokeSyscall();
            while (true) ;
        }

        public static Optional<Error> EmitInformation(string informationText)
        {
            return new Optional<Error>(Error.NotImplemented);
        }

        public static Optional<Error> EmitDebug(string debugText)
        {
            return new Optional<Error>(Error.NotImplemented);
        }

        public static Optional<Error> EmitWarning(string warningText)
        {
            return new Optional<Error>(Error.NotImplemented);
        }

        public static Optional<Error> EmitError(Error error, string errorText)
        {
            return new Optional<Error>(Error.NotImplemented);
        }

    }
}
