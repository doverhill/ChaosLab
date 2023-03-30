using System;
using System.Collections.Generic;

namespace Storm {
    internal class Output {
        private static object _lock = new object();
        public static ProcessEmitType KernelVerbosity = ProcessEmitType.Information;
        public static ProcessEmitType ProcessVerbosity = ProcessEmitType.Debug;

        private static Dictionary<bool, Dictionary<ProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>> HeadingColors =
            new Dictionary<bool, Dictionary<ProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>>
        {
            // kernel
            { true, new Dictionary<ProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>
                {
                { ProcessEmitType.Debug, (ConsoleColor.DarkBlue, ConsoleColor.White) },
                { ProcessEmitType.Information, (ConsoleColor.DarkBlue, ConsoleColor.White) },
                { ProcessEmitType.Warning, (ConsoleColor.DarkBlue, ConsoleColor.White) },
                { ProcessEmitType.Error, (ConsoleColor.DarkBlue, ConsoleColor.White) },
                }
            },
            // process
            { false, new Dictionary<ProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>
                {
                { ProcessEmitType.Debug, (ConsoleColor.Black, ConsoleColor.Cyan) },
                { ProcessEmitType.Information, (ConsoleColor.Black, ConsoleColor.Cyan) },
                { ProcessEmitType.Warning, (ConsoleColor.Black, ConsoleColor.Cyan) },
                { ProcessEmitType.Error, (ConsoleColor.Black, ConsoleColor.Cyan) },
                }
            }
        };
        private static Dictionary<bool, Dictionary<ProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>> TextColors =
            new Dictionary<bool, Dictionary<ProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>>
        {
            // kernel
            { true, new Dictionary<ProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>
                {
                { ProcessEmitType.Debug, (ConsoleColor.Black, ConsoleColor.DarkGray) },
                { ProcessEmitType.Information, (ConsoleColor.Black, ConsoleColor.White) },
                { ProcessEmitType.Warning, (ConsoleColor.Black, ConsoleColor.Yellow) },
                { ProcessEmitType.Error, (ConsoleColor.Black, ConsoleColor.Red) },
                }
            },
            // process
            { false, new Dictionary<ProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>
                {
                { ProcessEmitType.Debug, (ConsoleColor.Black, ConsoleColor.DarkGray) },
                { ProcessEmitType.Information, (ConsoleColor.Black, ConsoleColor.White) },
                { ProcessEmitType.Warning, (ConsoleColor.Black, ConsoleColor.Yellow) },
                { ProcessEmitType.Error, (ConsoleColor.Black, ConsoleColor.Red) },
                }
            }
        };

        private static void WriteLineForced(ProcessEmitType type, Process process, Process.Thread thread, bool isKernel, string format, object[] args = null) {
            lock (_lock) {
                var headingColors = HeadingColors[isKernel][type];
                Console.ForegroundColor = headingColors.Foreground;
                Console.BackgroundColor = headingColors.Background;

                var prefix = DateTime.Now.ToString("HH:mm:ss.fff") + " " + type.ToString().PadRight(11);
                var identifier = "";
                if (process != null) {
                    identifier = $"{process.ProcessId}:{thread.ThreadId}/{process.TrustChain}";
                }
                else {
                    identifier = "STORM";
                }
                Console.Write($"[{prefix} {identifier.PadRight(30)}]");
                Console.BackgroundColor = ConsoleColor.Black;
                Console.Write(" ");

                var textColor = TextColors[isKernel][type];
                Console.ForegroundColor = textColor.Foreground;
                Console.BackgroundColor = textColor.Background;

                Console.Write(format, args);

                Console.BackgroundColor = ConsoleColor.Black;
                Console.WriteLine();
            }
        }

        public static void WriteLineKernel(ProcessEmitType type, Process process, Process.Thread thread, string format, object[] args = null) {
            if (type > KernelVerbosity) return;
            WriteLineForced(type, process, thread, true, format, args);
        }

        public static void WriteLineProcess(ProcessEmitType type, Process process, Process.Thread thread, string format, object[] args = null) {
            if (type > ProcessVerbosity) return;
            WriteLineForced(type, process, thread, false, format, args);
        }
    }
}
