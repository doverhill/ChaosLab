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
        public static void ServiceCreate(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread)
        {
            var protocol = SyscallHelpers.ReadText(reader);
            var vendor = SyscallHelpers.ReadText(reader);
            var deviceName = SyscallHelpers.ReadText(reader);
            var deviceId = SyscallHelpers.ReadUuid(reader);

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ServiceCreate: protocol='" + protocol + "', vendor='" + vendor + "', deviceName='" + deviceName + "', deviceId=" + deviceId);
            var handle = Services.Create(process.ProcessId, protocol, vendor, deviceName, deviceId);

            writer.Write((int)Error.None);
            writer.Write(handle);
        }

        public static void ServiceDestroy(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread)
        {
            var serviceHandleId = reader.ReadUInt64();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ServiceDestroy: handleId=" + serviceHandleId);
            var success = Services.Destroy(process.ProcessId, serviceHandleId);

            if (success)
            {
                writer.Write((int)Error.None);
            }
            else
            {
                writer.Write((int)Error.PermissionDenied);
            }
        }

        public static void ServiceConnect(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread)
        {
            var protocol = SyscallHelpers.ReadText(reader);
            var vendor = SyscallHelpers.ReadText(reader);
            var deviceName = SyscallHelpers.ReadText(reader);
            var deviceId = SyscallHelpers.ReadUuid(reader);

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ServiceConnect: protocol='" + protocol + "', vendor='" + vendor + "', deviceName='" + deviceName + "', deviceId=" + deviceId);
            var service = Services.Lookup(protocol, vendor, deviceName, deviceId);

            if (service == null)
            {
                writer.Write((int)Error.NotFound);
            }
            else
            {
                var channelHandleId = Handles.Create(process.ProcessId, service.OwningPID, HandleType.Channel);
                Process.FireEvent(new Event(service.OwningPID, Error.None, service.HandleId, channelHandleId, HandleAction.ServiceConnected));

                writer.Write((int)Error.None);
                writer.Write(channelHandleId);
            }
        }

        public static void ChannelDestroy(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread)
        {
            var channelHandleId = reader.ReadUInt64();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ChannelDestroy: handleId=" + channelHandleId);
            var success = Handles.Destroy(process.ProcessId, channelHandleId);

            if (success)
            {
                writer.Write((int)Error.None);
            }
            else
            {
                writer.Write((int)Error.PermissionDenied);
            }
        }

        public static void ChannelSignal(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread)
        {
            var channelHandleId = reader.ReadUInt64();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ChannelSignal: handleId=" + channelHandleId);
            var handle = Handles.GetChannelHandleForSignal(channelHandleId, process.ProcessId);

            if (handle != null)
            {
                var receivingPID = handle.GetOtherProcessId(process.ProcessId);
                Process.FireEvent(new Event(receivingPID, Error.None, handle.Id, Handle.None, HandleAction.ChannelSignalled));

                writer.Write((int)Error.None);
            }
            else
            {
                writer.Write((int)Error.PermissionDenied);
            }
        }

        public static void EventWait(Socket socket, BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread)
        {
            var handle = SyscallHelpers.ReadOptionalU64(reader);
            var action = (HandleAction?)SyscallHelpers.ReadOptionalI32(reader);
            var timeoutMilliseconds = reader.ReadInt32();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL EventWait: handle=" + (handle.HasValue ? handle.Value : "any") + ", action=" + (action.HasValue ? action.ToString() : "any") + ", timeout=" + timeoutMilliseconds);
            var e = Events.Wait(socket, process, handle, action, timeoutMilliseconds);

            writer.Write((int)e.Error);
            if (e.Error == Error.None)
            {
                writer.Write(e.TargetHandle);
                writer.Write(e.ChannelHandle);
                writer.Write((int)e.Action);
            }
        }

        public static void ProcessSetInfo(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread)
        {
            var name = SyscallHelpers.ReadText(reader);

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ProcessSetInfo: name='" + name + "'");
            process.Name = name;

            writer.Write((int)Error.None);
        }

        public static void ProcessEmit(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread)
        {
            var emitType = (SyscallProcessEmitType)reader.ReadInt32();
            var error = (Error)reader.ReadInt32();
            var text = SyscallHelpers.ReadText(reader);

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ProcessEmit: type=" + emitType.ToString() + ", error=" + error + ", text='" + text + "'");
            if (emitType == SyscallProcessEmitType.Error)
            {
                Output.WriteLineProcess(emitType, process, thread, text + ": " + error.ToString());
            }
            else
            {
                Output.WriteLineProcess(emitType, process, thread, text);
            }

            writer.Write((int)Error.None);
        }
    }
}
