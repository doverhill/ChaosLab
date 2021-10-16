using System;
using System.Collections.Generic;

namespace Core
{
    public static class Process
    {
        private static List<Handle> handles;

        internal static void RegisterHandle(Handle handle)
        {
            handles.Add(handle);
        }

        public static Optional<Error> Run()
        {
            // main event loop
            while (true)
            {
                Syscalls.EventWait();
            }
            return new Optional<Error>(Error.NotImplemented);
        }

        public static void End()
        {
            EmitDebug("Process end");
            Syscalls.ProcessEnd();
            while (true) ;
        }

        public static Optional<Error> EmitInformation(string informationText)
        {
            var oldColor = Console.ForegroundColor;
            Console.ForegroundColor = ConsoleColor.Gray;
            Console.WriteLine("Information: " + informationText);
            Console.ForegroundColor = oldColor;

            return Syscalls.ProcessEmit(Storm.SyscallProcessEmitType.Information, Error.None, informationText);
        }

        public static Optional<Error> EmitDebug(string debugText)
        {
            var oldColor = Console.ForegroundColor;
            Console.ForegroundColor = ConsoleColor.Green;
            Console.WriteLine("Debug: " + debugText);
            Console.ForegroundColor = oldColor;

            return Syscalls.ProcessEmit(Storm.SyscallProcessEmitType.Debug, Error.None, debugText);
        }

        public static Optional<Error> EmitWarning(string warningText)
        {
            var oldColor = Console.ForegroundColor;
            Console.ForegroundColor = ConsoleColor.Yellow;
            Console.WriteLine("Warning: " + warningText);
            Console.ForegroundColor = oldColor;

            return Syscalls.ProcessEmit(Storm.SyscallProcessEmitType.Warning, Error.None, warningText);
        }

        public static Optional<Error> EmitError(Error error, string errorText)
        {
            var oldColor = Console.ForegroundColor;
            Console.ForegroundColor = ConsoleColor.Red;
            Console.WriteLine("Error: " + error.ToString() + ": " + errorText);
            Console.ForegroundColor = oldColor;

            return Syscalls.ProcessEmit(Storm.SyscallProcessEmitType.Error, error, errorText);
        }
    }
}
