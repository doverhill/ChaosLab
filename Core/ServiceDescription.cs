using System;

namespace Core
{
    public class ServiceDescription
    {
        public string Protocol;
        public Optional<string> Vendor;
        public Optional<string> DeviceName;
        public Optional<Guid> DeviceId;

        public ServiceDescription(string protocol, Optional<string> vendor, Optional<string> deviceName, Optional<Guid> deviceId)
        {
            Protocol = protocol;
            Vendor = vendor;
            DeviceName = deviceName;
            DeviceId = deviceId;
        }
    }
}
