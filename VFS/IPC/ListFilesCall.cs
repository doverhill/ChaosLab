using Core;
using System.Collections.Generic;

namespace VFS.IPC
{
    internal class ListFilesCall
    {
        internal struct ListFilesParameters
        {
            string DirectoryPath;
            bool Recursive;
        }


        public ErrorOr<IEnumerable<FileInfo>> ListFiles(ListFilesParameters parameters)
        {
        }
    }
}
