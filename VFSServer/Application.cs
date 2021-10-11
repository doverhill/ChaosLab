using Core;
using System;

namespace ListFiles
{
    public class Application
    {
        public void Start()
        {
            Process.EmitInformation("Starting VFS server");

            var serviceHandle = Service.Create(new ServiceDescription("vfs", new Optional<string>("Chaos"), new Optional<string>("Virtual file system server"), new Optional<Guid>(Guid.Empty))).Require("Failed to create service");
            Process.EmitDebug("Created VFS service handle " + serviceHandle.ToString());

            serviceHandle.OnConnect = HandleConnect;

            var run = Process.Run();
            if (run.HasValue()) Process.EmitError(run.Value(), "Failed to run application");

            Service.Destroy(serviceHandle);
            Process.End();
        }

        private void HandleConnect()
        {

        }
    }
}
