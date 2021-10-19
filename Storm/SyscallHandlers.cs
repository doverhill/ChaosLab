using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Uuids;

namespace Storm
{
    public enum SyscallNumber
    {
        Debug = 1,
        ServiceCreate = 2,
        ServiceConnect = 3,
        ServiceDestroy = 4,
        ChannelCreate = 5,
        ChannelDestroy = 6,
        EventWait = 7,
        ProcessCreate = 8,
        ProcessEmit = 9,
        ProcessDestroy = 10,
        ThreadCreate = 11,
        ThreadDestroy = 12
    }

    public enum SyscallProcessEmitType
    {
        Error = 1,
        Warning = 2,
        Information = 3,
        Debug = 4
    }

    internal static class SyscallHandlers
    {
        public static void ProcessEmit(BinaryWriter writer, ulong pid, SyscallProcessEmitType type, int error, string text)
        {
            Output.WriteLine(SyscallProcessEmitType.Debug, pid, "SYSCALL ProcessEmit: type=" + type.ToString() + ", error=" + error + ", text='" + text + "'");
            Output.WriteLineForced(type, pid, text);

            writer.Write((int)Error.None);
        }

        public static void ServiceCreate(BinaryWriter writer, ulong pid, string protocol, string vendor, string deviceName, Uuid? deviceId)
        {
            Output.WriteLine(SyscallProcessEmitType.Debug, pid, "SYSCALL ServiceCreate: protocol='" + protocol + "', vendor='" + vendor + "', deviceName='" + deviceName + "', deviceId=" + deviceId);
            var handle = Services.Create(pid, protocol, vendor, deviceName, deviceId);
            
            writer.Write((int)Error.None);
            writer.Write(handle);
        }

        public static void EventWait(BinaryWriter writer, ulong pid)
        {
            Output.WriteLine(SyscallProcessEmitType.Debug, pid, "SYSCALL EventWait");
            Thread.Sleep(5000);
            
            writer.Write((int)Error.None);
            writer.Write((ulong)666);
            writer.Write((int)HandleAction.Connect);
        }
    }
}
