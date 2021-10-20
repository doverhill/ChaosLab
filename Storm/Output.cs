using Core;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Storm
{
    internal class Output
    {
        public static SyscallProcessEmitType Verbosity = SyscallProcessEmitType.Information;

        public static void WriteLineForced(SyscallProcessEmitType type, Process process, string format, object[] args = null)
        {
            Console.ForegroundColor = ConsoleColor.Gray;
            if (process != null)
            {
                if (process.Name != null)
                {
                    Console.Write("[" + process.PID + "/" + process.Name + "] ");
                }
                else
                {
                    Console.Write("[" + process.PID + "] ");
                }
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

        public static void WriteLine(SyscallProcessEmitType type, Process process, string format, object[] args = null)
        {
            if (type > Verbosity) return;
            WriteLineForced(type, process, format, args);
        }
    }
}
