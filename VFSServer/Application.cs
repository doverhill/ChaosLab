using Core;
using System;
using System.IO.MemoryMappedFiles;
using System.Runtime.InteropServices;
using Uuids;

namespace VFSServer
{
    //unsafe struct TestStruct
    //{
    //    public int Signal;
    //    public char TestChar;
    //    public int TestInt;
    //}





    public class Application
    {
        public void Start()
        {
            Process.EmitInformation("Starting VFS server");

            //unsafe {
            //    var size = Marshal.SizeOf(typeof(TestStruct));
            //    Console.WriteLine("Open shared mem of size " + size);
            //    var sharedMemory = MemoryMappedFile.CreateOrOpen("test-chaos", size);
            //    var view = sharedMemory.CreateViewAccessor();
            //    byte* raw = null;
            //    view.SafeMemoryMappedViewHandle.AcquirePointer(ref raw);
            //    TestStruct* ipc = (TestStruct*)raw;
            //    Console.WriteLine("Got pointer " + (int)ipc);
            //    Console.WriteLine("Spin waiting...");
            //    while (ipc->Signal == 0) ;
            //    Console.WriteLine("Got char " + ipc->TestChar);
            //    Console.WriteLine("Got int " + ipc->TestInt);
            //    return;
            //}
            


            var serviceHandle = Service.Create("vfs", "Chaos", "Virtual file system server", Uuid.Empty).Require("Failed to create service");
            Process.EmitDebug("Created VFS service handle " + serviceHandle.ToString());

            serviceHandle.OnConnect = HandleConnect;

            var run = Process.Run();
            if (run.HasValue()) Process.EmitError(run.Value(), "Failed to run application");

            Service.Destroy(serviceHandle);
            Process.End();
        }

        private void HandleConnect()
        {
            Process.EmitDebug("Connect on service handle");
        }
    }
}
