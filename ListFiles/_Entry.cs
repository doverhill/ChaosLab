using Core;

namespace Chaos
{
    public static class Root
    {
        public static void Entry()
        {
            var application = new ListFiles.Application();
            application.Start();
            ASSERT.NotReached();
        }
    }
}
