using Core;

namespace VFS
{
    public static class Directory
    {
        public static ErrorOr<Enumerator<Item>> ListFiles(DirectoryPath path)
        {
            //var channel = Channel.Get();
            //channel.SendListFiles(path);
            //var result = await channel.ReceiveListFiles();
            return new ErrorOr<Enumerator<Item>>(Error.NotImplemented);
        }
    }
}
