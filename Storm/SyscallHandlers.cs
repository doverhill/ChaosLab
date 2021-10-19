using Core;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Uuids;

namespace Storm
{
    internal static class SyscallHandlers
    {
        public static void ProcessEmit(BinaryWriter writer, ulong pid, SyscallProcessEmitType type, Error error, string text)
        {
            Output.WriteLine(SyscallProcessEmitType.Debug, pid, "SYSCALL ProcessEmit: type=" + type.ToString() + ", error=" + error + ", text='" + text + "'");
            if (type == SyscallProcessEmitType.Error)
            {
                Output.WriteLineForced(type, pid, text + ": " + error.ToString());
            }
            else
            {
                Output.WriteLineForced(type, pid, text);
            }

            writer.Write((int)Error.None);
        }

        public static void ServiceCreate(BinaryWriter writer, ulong pid, string protocol, string vendor, string deviceName, Uuid? deviceId)
        {
            Output.WriteLine(SyscallProcessEmitType.Debug, pid, "SYSCALL ServiceCreate: protocol='" + protocol + "', vendor='" + vendor + "', deviceName='" + deviceName + "', deviceId=" + deviceId);
            var handle = Services.Create(pid, protocol, vendor, deviceName, deviceId);
            
            writer.Write((int)Error.None);
            writer.Write(handle);
        }

        public static void ServiceConnect(BinaryWriter writer, ulong pid, string protocol, string vendor, string deviceName, Uuid? deviceId)
        {
            Output.WriteLine(SyscallProcessEmitType.Debug, pid, "SYSCALL ServiceConnect: protocol='" + protocol + "', vendor='" + vendor + "', deviceName='" + deviceName + "', deviceId=" + deviceId);
            var service = Services.Lookup(protocol, vendor, deviceName, deviceId);

            if (service == null)
            {
                writer.Write((int)Error.NotFound);
            }
            else
            {
                Events.Fire(new Event(service.OwningPID, Error.None, service.Handle, HandleAction.Connect));

                var handle = Handles.AllocateHandle(pid, HandleType.ServiceConnection);
                writer.Write((int)Error.None);
                writer.Write(handle);
            }
        }

        public static void EventWait(BinaryWriter writer, ulong pid, int timeoutMilliseconds)
        {
            Output.WriteLine(SyscallProcessEmitType.Debug, pid, "SYSCALL EventWait");
            var e = Events.Wait(pid, timeoutMilliseconds);

            writer.Write((int)e.Error);
            if (e.Error == Error.None)
            {
                writer.Write(e.Handle);
                writer.Write((int)e.Action);
            }
        }
    }
}
