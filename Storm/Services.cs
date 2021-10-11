using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Storm
{
    internal class Service
    {
        public ulong OwningPID;
        public string Protocol;
        public string Vendor;
        public string DeviceName;
        public Guid DeviceId;

        public Service(ulong owningPID, string protocol, string vendor, string deviceName, Guid deviceId)
        {
            OwningPID = owningPID;
            Protocol = protocol;
            Vendor = vendor;
            DeviceName = deviceName;
            DeviceId = deviceId;
        }
    }

    internal static class Services
    {
        private static object _lock = new object();
        private static Dictionary<string, List<Service>> services = new Dictionary<string, List<Service>>();

        public static ulong Create(ulong pid, string protocol, string vendor, string deviceName, Guid? deviceId)
        {
            var handle = Handles.AllocateHandle(pid, HandleType.Service);
            var service = new Service(pid, protocol, vendor, deviceName, deviceId.Value);
            lock (_lock)
            {
                List<Service> list;
                if (!services.TryGetValue(protocol, out list))
                {
                    list = new List<Service>();
                    services[protocol] = list;
                }
                list.Add(service);
            }

            return handle;
        }
    }
}
