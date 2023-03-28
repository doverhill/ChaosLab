namespace Storm {
    public class StartupCommand
    {
        public int DelayMs;
        public string Path;
        public string Executable;

        public StartupCommand(int delayMs, string path, string executable)
        {
            DelayMs = delayMs;
            Path = path;
            Executable = executable;
        }
    }
}
