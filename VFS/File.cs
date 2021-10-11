using Core;

namespace VFS
{
    public static class File
    {
        public static ErrorOr<FileHandle> Open(FilePath path)
        {
            return new ErrorOr<FileHandle>(Error.NotImplemented);
        }

        public static Optional<Error> CloseFile(FileHandle handle)
        {
            return new Optional<Error>(Error.NotImplemented);
        }
    }
}
