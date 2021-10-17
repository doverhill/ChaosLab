using Core;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace VFS
{
    public static class VFSClient
    {
        public static Optional<Error> Initialize()
        {
            // Connect to VFS service
            var handle = Service.Connect()
        }
    }
}
