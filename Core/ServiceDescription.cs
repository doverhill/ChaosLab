using System;

namespace Core
{
    public class ServiceDescription
    {
        private string protocol;
        private Optional<string> vendor;
        private Optional<string> deviceName;
        private Optional<Guid> deviceId;

        public ServiceDescription(string protocol, Optional<string> vendor, Optional<string> deviceName, Optional<Guid> deviceId)
        {
            this.protocol = protocol;
            this.vendor = vendor;
            this.deviceName = deviceName;
            this.deviceId = deviceId;
        }
    }
}
