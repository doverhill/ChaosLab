using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Uuids;

namespace Storm
{
    internal class Service
    {
        public ulong OwningPID;
        public ulong Handle;
        public string Protocol;
        public string Vendor;
        public string DeviceName;
        public Uuid DeviceId;

        public Service(ulong owningPID, ulong handle, string protocol, string vendor, string deviceName, Uuid deviceId)
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

        public static ulong Create(ulong pid, string protocol, string vendor, string deviceName, Uuid? deviceId)
        {
            var handle = Handles.AllocateHandle(pid, HandleType.Service);
            var service = new Service(pid, handle, protocol, vendor, deviceName, deviceId.Value);
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

        public static Service Lookup(string protocol, string vendor, string deviceName, Uuid? deviceId)
        {
            lock (_lock)
            {
                if (!services.TryGetValue(protocol, out var serviceList))
                {
                    return null;
                }

                foreach (var service in serviceList)
                {
                    if (vendor != null && service.Vendor != vendor) continue;
                    if (deviceName != null && service.DeviceName != deviceName) continue;
                    if (deviceId.HasValue && service.DeviceId != deviceId.Value) continue;
                    return service;
                }

                return null;
            }
        }

        public static void CleanupProcess(ulong pid)
        {
            lock (_lock)
            {
                foreach (var serviceList in services.Values)
                {
                    serviceList.RemoveAll(s => s.OwningPID == pid);
                }
            }
        }
    }
}
