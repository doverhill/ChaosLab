using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Storm
{
    public enum SyscallNumber
    {
        Debug,
        ServiceCreate,
        ServiceConnect,
        ServiceDestroy,
        ChannelCreate,
        ChannelDestroy,
        EventWait,
        ProcessCreate,
        ProcessEmit,
        ProcessDestroy,
        ThreadCreate,
        ThreadDestroy
    }

    public enum SyscallProcessEmitType
    {
        Information,
        Debug,
        Warning,
        Error
    }

    internal static class SyscallHandlers
    {
        public static void ProcessEmit(BinaryWriter writer, ulong pid, SyscallProcessEmitType type, int error, string text)
        {
            Console.WriteLine("[" + pid + "] SYSCALL ProcessEmit: type=" + type.ToString() + ", error=" + error + ", text='" + text + "'");
            writer.Write((int)Error.None);
        }

        public static void ServiceCreate(BinaryWriter writer, ulong pid, string protocol, string vendor, string deviceName, Guid? deviceId)
        {
            Console.WriteLine("[" + pid + "] SYSCALL ServiceCreate: protocol='" + protocol + "', vendor='" + vendor + "', deviceName='" + deviceName + "', deviceId=" + deviceId);
            var handle = Services.Create(pid, protocol, vendor, deviceName, deviceId);
            writer.Write((int)Error.None);
            writer.Write(handle);
        }
    }
}
