using Core;
using System.Collections.Generic;

namespace VFS.IPC
{
    internal class GetFileInfoCall
    {
        internal struct GetFileInfoParameters
        {
            string FilePath;
        }

        internal struct GetFileInfoReturn
        {
            FileInfo File;
        }

        public ErrorOr<GetFileInfoReturn> GetFileInfo(GetFileInfoParameters parameters)
        {
            return new ErrorOr<GetFileInfoReturn>(Error.NotImplemented);
        }
    }
}
