using Core;

namespace Chaos
{
    public static class Root
    {
        public static void Entry()
        {
            var application = new VFSServer.Application();
            application.Start();
            ASSERT.NotReached();
        }
    }
}
