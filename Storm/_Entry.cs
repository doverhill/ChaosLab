namespace Chaos
{
    public static class Root
    {
        public static void Entry()
        {
            var kernel = new Storm.Kernel();
            kernel.Start();
        }
    }
}
