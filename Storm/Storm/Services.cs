using System.Collections.Generic;
using System.Diagnostics;
using Uuids;

namespace Storm {
    internal class Service {
        public ulong OwningProcessId;
        public ulong HandleId;
        public string Protocol;
        public string Owner;
        public Uuid DeviceId;

        public Service(ulong owningProcessId, ulong handleId, string protocol, string owner, Uuid deviceId) {
            OwningProcessId = owningProcessId;
            HandleId = handleId;
            Protocol = protocol;
            Owner = owner;
            DeviceId = deviceId;
        }
    }

    internal class ServiceSubscription {
        public ulong OwningProcessId;
        public ulong HandleId;
        public string Protocol;
        public string Owner;
        public Uuid? DeviceId;

        public ServiceSubscription(ulong owningProcessId, ulong handleId, string protocol, string owner, Uuid? deviceId) {
            OwningProcessId = owningProcessId;
            HandleId = handleId;
            Protocol = protocol;
            Owner = owner;
            DeviceId = deviceId;
        }
    }

    internal static class Services {
        private static object _lock = new();
        private static Dictionary<string, List<Service>> _services = new();
        private static Dictionary<string, List<ServiceSubscription>> _serviceSubscriptions = new();

        public static ErrorOr<ulong> Create(Process process, string protocol, string owner, Uuid deviceId) {
            var handle = Handles.Create(process.ProcessId, Handle.Type.Service);
            var service = new Service(process.ProcessId, handle, protocol, owner, deviceId);

            lock (_lock) {
                if (!_services.TryGetValue(protocol, out var list)) {
                    list = new();
                    _services[protocol] = list;
                }
                list.Add(service);
            }

            return ErrorOr<ulong>.Ok(handle);
        }

        public static ErrorOr<ulong> CreateSubscription(Process process, string protocol, string owner, Uuid? deviceId) {
            var handle = Handles.Create(process.ProcessId, Handle.Type.ServiceSubscribe);
            var subscription = new ServiceSubscription(process.ProcessId, handle, protocol, owner, deviceId);

            lock (_lock) {
                if (!_serviceSubscriptions.TryGetValue(protocol, out var list)) {
                    list = new();
                    _serviceSubscriptions[protocol] = list;
                }
                list.Add(subscription);
            }

            // FIXME can we have a condition where lookup returns nothing and a service create runs at the same time before adding so that this process never gets notified???
            var service = Lookup(protocol, owner, deviceId);
            if (service.HasValue) {
                // FIXME do the actual connect here, creating a channel and two events to both processes
                process.PostServiceAvailableEvent(handle);
            }

            return ErrorOr<ulong>.Ok(handle);
        }

        //public static bool Destroy(ulong PID, ulong serviceHandleId)
        //{
        //    lock (_lock)
        //    {
        //        foreach (var list in services.Values)
        //        {
        //            int didRemove = list.RemoveAll(s => s.OwningProcessId == PID && s.HandleId == serviceHandleId);
        //            return didRemove == 1;
        //        }
        //    }

        //    return false;
        //}

        private static Optional<Service> Lookup(string protocol, string owner, Uuid? deviceId) {
            lock (_lock) {
                if (!_services.TryGetValue(protocol, out var serviceList)) {
                    return Optional<Service>.None();
                }

                foreach (var service in serviceList) {
                    if (owner != null && service.Owner != owner) continue;
                    if (deviceId.HasValue && service.DeviceId != deviceId.Value) continue;
                    return Optional<Service>.WithValue(service);
                }

                return Optional<Service>.None();
            }
        }

        //public static void Cleanup(Process process)
        //{
        //    lock (_lock)
        //    {
        //        foreach (var serviceList in services.Values)
        //        {
        //            serviceList.RemoveAll(s => s.OwningProcessId == process.ProcessId);
        //        }
        //    }
        //}
    }
}
