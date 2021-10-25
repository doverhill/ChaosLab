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
        public ulong HandleId;
        public string Protocol;
        public string Vendor;
        public string DeviceName;
        public Uuid DeviceId;

        public Service(ulong owningPID, ulong handleId, string protocol, string vendor, string deviceName, Uuid deviceId)
        {
            OwningPID = owningPID;
            HandleId = handleId;
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

        public static ulong Create(ulong PID, string protocol, string vendor, string deviceName, Uuid? deviceId)
        {
            var handle = Handles.Create(PID, HandleType.Service);
            var service = new Service(PID, handle, protocol, vendor, deviceName, deviceId.Value);

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

        public static bool Destroy(ulong PID, ulong serviceHandleId)
        {
            lock (_lock)
            {
                foreach (var list in services.Values)
                {
                    int didRemove = list.RemoveAll(s => s.OwningPID == PID && s.HandleId == serviceHandleId);
                    return didRemove == 1;
                }
            }

            return false;
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

        public static void CleanupProcess(ulong PID)
        {
            lock (_lock)
            {
                foreach (var serviceList in services.Values)
                {
                    serviceList.RemoveAll(s => s.OwningPID == PID);
                }
            }
        }
    }
}
