using Core;
using System;
using System.IO.MemoryMappedFiles;
using System.Runtime.InteropServices;
using VFS;

namespace ListFiles
{
    public class Application
    {
        public void Start()
        {
            Process.SetInfo("DirectoryList (.net)");

            var channel = Service.Connect("vfs", null, null, null).Require("Faild to connect to service");
            Process.EmitDebug(string.Format("Connected to service, got channel {0}", channel));

            List(channel);
        }

        private void List(Channel channel)
        {
            unsafe
            {
                byte* raw = channel.GetRawPointer();
                while (*raw == 0) ;
                var result = *raw;
                Process.EmitInformation("Got " + result + " from server");
            }
        }
    }
}
