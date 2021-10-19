using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Storm
{
    internal class Output
    {
        public static SyscallProcessEmitType Verbosity = SyscallProcessEmitType.Debug;

        public static void WriteLineForced(SyscallProcessEmitType type, ulong? pid, string format, object[] args = null)
        {
            Console.ForegroundColor = ConsoleColor.Gray;
            if (pid.HasValue)
            {
                Console.Write("[" + pid.Value + "] ");
            }
            else
            {
                Console.Write("[KERNEL] ");
            }

            switch (type)
            {
                case SyscallProcessEmitType.Information:
                    Console.ForegroundColor = ConsoleColor.Gray;
                    break;

                case SyscallProcessEmitType.Warning:
                    Console.ForegroundColor = ConsoleColor.Yellow;
                    break;

                case SyscallProcessEmitType.Debug:
                    Console.ForegroundColor = ConsoleColor.DarkGray;
                    break;

                case SyscallProcessEmitType.Error:
                    Console.ForegroundColor = ConsoleColor.Red;
                    break;
            }

            Console.WriteLine(format, args);
        }

        public static void WriteLine(SyscallProcessEmitType type, ulong? pid, string format, object[] args = null)
        {
            if (type > Verbosity) return;
            WriteLineForced(type, pid, format, args);
        }
    }
}
