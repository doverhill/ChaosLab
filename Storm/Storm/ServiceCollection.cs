using System.Collections.Generic;
using Uuids;

namespace Storm {
    internal static class ServiceCollection {
        private static object _lock = new();
        private static Dictionary<string, List<Service>> _services = new();
        private static Dictionary<string, List<ServiceSubscription>> _serviceSubscriptions = new();

        public static ErrorOr<ulong> Create(Process process, string protocol, string owner, Uuid deviceId) {
            var handle = HandleCollection.Create(process.ProcessId, Handle.HandleType.Service);
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
            var handle = HandleCollection.Create(process.ProcessId, Handle.HandleType.ServiceSubscribe);
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
                process.PostServiceAvailableEvent(handle, channelHandleId);
            }

            return ErrorOr<ulong>.Ok(handle);
        }

        public static void Remove(ulong serviceHandleId) {
            lock (_lock) {
                foreach (var list in _services.Values) {
                    list.RemoveAll(s => s.HandleId == serviceHandleId);
                }
            }
        }

        public static void RemoveSubscription(ulong subscriptionHandleId) {
            lock (_lock) {
                foreach (var list in _serviceSubscriptions.Values) {
                    list.RemoveAll(s => s.HandleId == subscriptionHandleId);
                }
            }
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
