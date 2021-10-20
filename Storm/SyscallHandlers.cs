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
        public static void ProcessEmit(BinaryWriter writer, Process process, SyscallProcessEmitType type, Error error, string text)
        {
            Output.WriteLine(SyscallProcessEmitType.Debug, process, "SYSCALL ProcessEmit: type=" + type.ToString() + ", error=" + error + ", text='" + text + "'");
            if (type == SyscallProcessEmitType.Error)
            {
                Output.WriteLineForced(type, process, text + ": " + error.ToString());
            }
            else
            {
                Output.WriteLineForced(type, process, text);
            }

            writer.Write((int)Error.None);
        }

        public static void ServiceCreate(BinaryWriter writer, Process process, string protocol, string vendor, string deviceName, Uuid? deviceId)
        {
            Output.WriteLine(SyscallProcessEmitType.Debug, process, "SYSCALL ServiceCreate: protocol='" + protocol + "', vendor='" + vendor + "', deviceName='" + deviceName + "', deviceId=" + deviceId);
            var handle = Services.Create(process.PID, protocol, vendor, deviceName, deviceId);
            
            writer.Write((int)Error.None);
            writer.Write(handle);
        }

        public static void ServiceConnect(BinaryWriter writer, Process process, string protocol, string vendor, string deviceName, Uuid? deviceId)
        {
            Output.WriteLine(SyscallProcessEmitType.Debug, process, "SYSCALL ServiceConnect: protocol='" + protocol + "', vendor='" + vendor + "', deviceName='" + deviceName + "', deviceId=" + deviceId);
            var service = Services.Lookup(protocol, vendor, deviceName, deviceId);

            if (service == null)
            {
                writer.Write((int)Error.NotFound);
            }
            else
            {
                var channelHandle = Handles.AllocateHandle(process.PID, HandleType.Channel);
                Events.Fire(new Event(service.OwningPID, Error.None, service.Handle, channelHandle, HandleAction.Connect));

                writer.Write((int)Error.None);
                writer.Write(channelHandle);
            }
        }

        public static void EventWait(BinaryWriter writer, Process process, int timeoutMilliseconds)
        {
            Output.WriteLine(SyscallProcessEmitType.Debug, process, "SYSCALL EventWait");
            var e = Events.Wait(process.PID, timeoutMilliseconds);

            writer.Write((int)e.Error);
            if (e.Error == Error.None)
            {
                writer.Write(e.TargetHandle);
                writer.Write(e.ArgumentHandle);
                writer.Write((int)e.Action);
            }
        }
    }
}
