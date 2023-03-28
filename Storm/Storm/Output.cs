namespace Storm {
    internal class Output
    {
        private static object _lock = new object();
        public static SyscallProcessEmitType KernelVerbosity = SyscallProcessEmitType.Information;
        public static SyscallProcessEmitType ProcessVerbosity = SyscallProcessEmitType.Debug;

        private static Dictionary<bool, Dictionary<SyscallProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>> HeadingColors =
            new Dictionary<bool, Dictionary<SyscallProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>>
        {
            // kernel
            { true, new Dictionary<SyscallProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>
                {
                { SyscallProcessEmitType.Debug, (ConsoleColor.DarkBlue, ConsoleColor.White) },
                { SyscallProcessEmitType.Information, (ConsoleColor.DarkBlue, ConsoleColor.White) },
                { SyscallProcessEmitType.Warning, (ConsoleColor.DarkBlue, ConsoleColor.White) },
                { SyscallProcessEmitType.Error, (ConsoleColor.DarkBlue, ConsoleColor.White) },
                }
            },
            // process
            { false, new Dictionary<SyscallProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>
                {
                { SyscallProcessEmitType.Debug, (ConsoleColor.Black, ConsoleColor.Cyan) },
                { SyscallProcessEmitType.Information, (ConsoleColor.Black, ConsoleColor.Cyan) },
                { SyscallProcessEmitType.Warning, (ConsoleColor.Black, ConsoleColor.Cyan) },
                { SyscallProcessEmitType.Error, (ConsoleColor.Black, ConsoleColor.Cyan) },
                }
            }
        };
        private static Dictionary<bool, Dictionary<SyscallProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>> TextColors =
            new Dictionary<bool, Dictionary<SyscallProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>>
        {
            // kernel
            { true, new Dictionary<SyscallProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>
                {
                { SyscallProcessEmitType.Debug, (ConsoleColor.Black, ConsoleColor.DarkGray) },
                { SyscallProcessEmitType.Information, (ConsoleColor.Black, ConsoleColor.White) },
                { SyscallProcessEmitType.Warning, (ConsoleColor.Black, ConsoleColor.Yellow) },
                { SyscallProcessEmitType.Error, (ConsoleColor.Black, ConsoleColor.Red) },
                }
            },
            // process
            { false, new Dictionary<SyscallProcessEmitType, (ConsoleColor Background, ConsoleColor Foreground)>
                {
                { SyscallProcessEmitType.Debug, (ConsoleColor.Black, ConsoleColor.DarkGray) },
                { SyscallProcessEmitType.Information, (ConsoleColor.Black, ConsoleColor.White) },
                { SyscallProcessEmitType.Warning, (ConsoleColor.Black, ConsoleColor.Yellow) },
                { SyscallProcessEmitType.Error, (ConsoleColor.Black, ConsoleColor.Red) },
                }
            }
        };

        private static void WriteLineForced(SyscallProcessEmitType type, Process process, Process.Thread thread, bool isKernel, string format, object[] args = null)
        {
            lock (_lock)
            {
                var headingColors = HeadingColors[isKernel][type];
                Console.ForegroundColor = headingColors.Foreground;
                Console.BackgroundColor = headingColors.Background;

                var prefix = DateTime.Now.ToString("HH:mm:ss.fff") + " " + type.ToString().PadRight(11);
                var identifier = "";
                if (process != null)
                {
                    if (!string.IsNullOrEmpty(process.Name))
                    {
                        identifier = $"{process.ProcessId}:{thread.ThreadId}/{process.Name}";
                    }
                    else
                    {
                        identifier = $"{process.ProcessId}:{thread.ThreadId}";
                    }
                }
                else
                {
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

        public static void WriteLineKernel(SyscallProcessEmitType type, Process process, Process.Thread thread, string format, object[] args = null)
        {
            if (type > KernelVerbosity) return;
            WriteLineForced(type, process, thread, true, format, args);
        }

        public static void WriteLineProcess(SyscallProcessEmitType type, Process process, Process.Thread thread, string format, object[] args = null)
        {
            if (type > ProcessVerbosity) return;
            WriteLineForced(type, process, thread, false, format, args);
        }
    }
}
