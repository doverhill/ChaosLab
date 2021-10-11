using Core;

namespace ListFiles
{
    public static class _Entry
    {
        public static void ApplicationEntry()
        {
            var application = new Application();
            application.Start();
            ASSERT.NotReached();
        }
    }
}
