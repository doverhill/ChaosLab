using Core;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Net.Sockets;
using System.Text;
using System.Threading.Tasks;
using Uuids;

namespace Storm
{
    internal static class SyscallHandlers
    {
        public static void ServiceCreate(BinaryReader reader, BinaryWriter writer, Process process)
        {
            var protocol = SyscallHelpers.ReadText(reader);
            var vendor = SyscallHelpers.ReadText(reader);
            var deviceName = SyscallHelpers.ReadText(reader);
            var deviceId = SyscallHelpers.ReadUuid(reader);

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, "SYSCALL ServiceCreate: protocol='" + protocol + "', vendor='" + vendor + "', deviceName='" + deviceName + "', deviceId=" + deviceId);
            var handle = Services.Create(process.PID, protocol, vendor, deviceName, deviceId);

            writer.Write((int)Error.None);
            writer.Write(handle);
        }

        public static void ServiceDestroy(BinaryReader reader, BinaryWriter writer, Process process)
        {
            var serviceHandleId = reader.ReadUInt64();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, "SYSCALL ServiceDestroy: handleId=" + serviceHandleId);
            var success = Services.Destroy(process.PID, serviceHandleId);

            if (success)
            {
                writer.Write((int)Error.None);
            }
            else
            {
                writer.Write((int)Error.PermissionDenied);
            }
        }

        public static void ServiceConnect(BinaryReader reader, BinaryWriter writer, Process process)
        {
            var protocol = SyscallHelpers.ReadText(reader);
            var vendor = SyscallHelpers.ReadText(reader);
            var deviceName = SyscallHelpers.ReadText(reader);
            var deviceId = SyscallHelpers.ReadUuid(reader);

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, "SYSCALL ServiceConnect: protocol='" + protocol + "', vendor='" + vendor + "', deviceName='" + deviceName + "', deviceId=" + deviceId);
            var service = Services.Lookup(protocol, vendor, deviceName, deviceId);

            if (service == null)
            {
                writer.Write((int)Error.NotFound);
            }
            else
            {
                var channelHandleId = Handles.Create(process.PID, service.OwningPID, HandleType.Channel);
                Events.Fire(new Event(service.OwningPID, Error.None, service.HandleId, channelHandleId, HandleAction.ServiceConnected, 0));

                writer.Write((int)Error.None);
                writer.Write(channelHandleId);
            }
        }

        public static void ChannelDestroy(BinaryReader reader, BinaryWriter writer, Process process)
        {
            var channelHandleId = reader.ReadUInt64();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, "SYSCALL ChannelDestroy: handleId=" + channelHandleId);
            var success = Handles.Destroy(process.PID, channelHandleId);

            if (success)
            {
                writer.Write((int)Error.None);
            }
            else
            {
                writer.Write((int)Error.PermissionDenied);
            }
        }

        public static void ChannelMessage(BinaryReader reader, BinaryWriter writer, Process process)
        {
            var channelHandleId = reader.ReadUInt64();
            var message = reader.ReadUInt64();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, "SYSCALL ChannelSignal: handleId=" + channelHandleId + ", message=" + message);
            var handle = Handles.GetChannelHandleForSignal(channelHandleId, process.PID);

            if (handle != null)
            {
                var receivingPID = handle.GetOtherPID(process.PID);
                Events.Fire(new Event(receivingPID, Error.None, handle.Id, Handle.None, HandleAction.ChannelMessaged, message));

                writer.Write((int)Error.None);
            }
            else
            {
                writer.Write((int)Error.PermissionDenied);
            }
        }

        public static void EventWait(Socket socket, BinaryReader reader, BinaryWriter writer, Process process)
        {
            var timeoutMilliseconds = reader.ReadInt32();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, "SYSCALL EventWait: timeout=" + timeoutMilliseconds);
            var e = Events.Wait(socket, process.PID, timeoutMilliseconds);

            writer.Write((int)e.Error);
            if (e.Error == Error.None)
            {
                writer.Write(e.TargetHandle);
                writer.Write(e.ArgumentHandle);
                writer.Write((int)e.Action);
                writer.Write(e.Parameter);
            }
        }

        public static void ProcessSetInfo(BinaryReader reader, BinaryWriter writer, Process process)
        {
            var name = SyscallHelpers.ReadText(reader);

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, "SYSCALL ProcessSetInfo: name='" + name + "'");
            process.Name = name;

            writer.Write((int)Error.None);
        }

        public static void ProcessEmit(BinaryReader reader, BinaryWriter writer, Process process)
        {
            var emitType = (SyscallProcessEmitType)reader.ReadInt32();
            var error = (Error)reader.ReadInt32();
            var text = SyscallHelpers.ReadText(reader);

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, "SYSCALL ProcessEmit: type=" + emitType.ToString() + ", error=" + error + ", text='" + text + "'");
            if (emitType == SyscallProcessEmitType.Error)
            {
                Output.WriteLineProcess(emitType, process, text + ": " + error.ToString());
            }
            else
            {
                Output.WriteLineProcess(emitType, process, text);
            }

            writer.Write((int)Error.None);
        }
    }
}
