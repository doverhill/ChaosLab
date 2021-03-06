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
        private static object _lock = new object();
        public static SyscallProcessEmitType KernelVerbosity = SyscallProcessEmitType.Information;
        public static SyscallProcessEmitType ProcessVerbosity = SyscallProcessEmitType.Debug;

        private static void WriteLineForced(SyscallProcessEmitType type, Process process, string format, object[] args = null)
        {
            lock (_lock)
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
        }

        public static void WriteLineKernel(SyscallProcessEmitType type, Process process, string format, object[] args = null)
        {
            if (type > KernelVerbosity) return;
            WriteLineForced(type, process, format, args);
        }

        public static void WriteLineProcess(SyscallProcessEmitType type, Process process, string format, object[] args = null)
        {
            if (type > ProcessVerbosity) return;
            WriteLineForced(type, process, format, args);
        }
    }
}
