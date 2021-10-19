using Core;
using System;
using System.IO.MemoryMappedFiles;
using System.Runtime.InteropServices;
using VFS;

namespace ListFiles
{
    unsafe struct TestStruct
    {
        public int Signal;
        public char TestChar;
        public int TestInt;
    }



    public class Application
    {
        public void Start()
        {
            //unsafe
            //{
            //    var size = Marshal.SizeOf(typeof(TestStruct));
            //    Console.WriteLine("Open shared mem of size " + size);
            //    var sharedMemory = MemoryMappedFile.CreateOrOpen("test-chaos", size);
            //    var view = sharedMemory.CreateViewAccessor();
            //    byte* raw = null;
            //    view.SafeMemoryMappedViewHandle.AcquirePointer(ref raw);
            //    TestStruct* ipc = (TestStruct*)raw;
            //    ipc->TestChar = 'x';
            //    ipc->TestInt = 42;
            //    ipc->Signal = 1;
            //    return;
            //}

            VFSClient.Initialize();


            var files = Directory.ListFiles(new DirectoryPath("/"), false).Require("Could not list files");

            var result = files.ForEach((item) => {
                //Process.EmitDebug(item.Path.ToString());
                return false;
            });

            if (result.HasValue()) Process.EmitError(result.Value(), "Could not iterate files");

            Process.End();
        }
    }
}
