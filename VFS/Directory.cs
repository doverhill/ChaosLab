using Core;
using System.Collections.Generic;
using VFS.IPC;

namespace VFS
{
    public static class Directory
    {
        public static ErrorOr<Enumerator<FileInfo>> ListFiles(DirectoryPath path, bool recursive)
        {
            //var channel = Channel.Get();
            //channel.SendListFiles(path);
            //var result = await channel.ReceiveListFiles();
            return new ErrorOr<Enumerator<Item>>(Error.NotImplemented);
        }
    }
}
