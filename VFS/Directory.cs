using Core;

namespace VFS
{
    public static class Directory
    {
        public static ErrorOr<Enumerator<Item>> ListFiles(DirectoryPath path)
        {
            return new ErrorOr<Enumerator<Item>>(Error.NotImplemented);
        }
    }
}
