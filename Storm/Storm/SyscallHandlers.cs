using System.Net.Sockets;

namespace Storm {
    internal static class SyscallHandlers {
        public static void ServiceCreate(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var protocol = SyscallHelpers.ReadText(reader);
            var vendor = SyscallHelpers.ReadText(reader);
            var deviceName = SyscallHelpers.ReadText(reader);
            var deviceId = SyscallHelpers.ReadUuid(reader);

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ServiceCreate: protocol='" + protocol + "', vendor='" + vendor + "', deviceName='" + deviceName + "', deviceId=" + deviceId);
            var handle = Services.Create(process.ProcessId, protocol, vendor, deviceName, deviceId);

            writer.Write((int)Error.None);
            writer.Write(handle);
        }

        public static void ServiceDestroy(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var serviceHandleId = reader.ReadUInt64();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ServiceDestroy: handleId=" + serviceHandleId);
            var success = Services.Destroy(process.ProcessId, serviceHandleId);

            if (success) {
                writer.Write((int)Error.None);
            }
            else {
                writer.Write((int)Error.PermissionDenied);
            }
        }

        public static void ServiceSubscribe(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            // FIXME: rewrite this
            ASSERT.NotReached();

            var protocol = SyscallHelpers.ReadText(reader);
            var vendor = SyscallHelpers.ReadText(reader);
            var deviceName = SyscallHelpers.ReadText(reader);
            var deviceId = SyscallHelpers.ReadUuid(reader);

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ServiceConnect: protocol='" + protocol + "', vendor='" + vendor + "', deviceName='" + deviceName + "', deviceId=" + deviceId);
            var service = Services.Lookup(protocol, vendor, deviceName, deviceId);

            if (service == null) {
                writer.Write((int)Error.NotFound);
            }
            else {
                var channelHandleId = Handles.Create(process.ProcessId, service.OwningPID, HandleType.Channel);
                Process.FireEvent(new Event(service.OwningPID, Error.None, service.HandleId, channelHandleId, HandleAction.ServiceConnected));

                writer.Write((int)Error.None);
                writer.Write(channelHandleId);
            }
        }

        public static void ChannelDestroy(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var channelHandleId = reader.ReadUInt64();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ChannelDestroy: handleId=" + channelHandleId);
            var success = Handles.Destroy(process.ProcessId, channelHandleId);

            if (success) {
                writer.Write((int)Error.None);
            }
            else {
                writer.Write((int)Error.PermissionDenied);
            }
        }

        public static void ChannelSignal(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var channelHandleId = reader.ReadUInt64();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ChannelSignal: handleId=" + channelHandleId);
            var handle = Handles.GetChannelHandleForSignal(channelHandleId, process.ProcessId);

            if (handle != null) {
                // FIXME only one signal event should be allowed per handle, they should never queue up
                var receivingPID = handle.GetOtherProcessId(process.ProcessId);
                Process.FireEvent(new Event(receivingPID, Error.None, handle.Id, Handle.None, HandleAction.ChannelSignalled));

                writer.Write((int)Error.None);
            }
            else {
                writer.Write((int)Error.PermissionDenied);
            }
        }

        public static void EventWait(Socket socket, BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var handle = SyscallHelpers.ReadOptionalU64(reader);
            var action = (HandleAction?)SyscallHelpers.ReadOptionalI32(reader);
            var timeoutMilliseconds = reader.ReadInt32();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL EventWait: handle=" + (handle.HasValue ? handle.Value : "any") + ", action=" + (action.HasValue ? action.ToString() : "any") + ", timeout=" + timeoutMilliseconds);
            var e = Events.Wait(socket, process, handle, action, timeoutMilliseconds);

            writer.Write((int)e.Error);
            if (e.Error == Error.None) {
                writer.Write(e.TargetHandle);
                writer.Write(e.ChannelHandle);
                writer.Write((int)e.Action);
            }
        }

        //public static void ProcessSetInfo(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread)
        //{
        //    var name = SyscallHelpers.ReadText(reader);

        //    Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ProcessSetInfo: name='" + name + "'");
        //    process.Name = name;

        //    writer.Write((int)Error.None);
        //}

        public static void ProcessCreate(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {

        }

        public static void ProcessEmit(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var emitType = (SyscallProcessEmitType)reader.ReadInt32();
            var error = (Error)reader.ReadInt32();
            var text = SyscallHelpers.ReadText(reader);

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ProcessEmit: type=" + emitType.ToString() + ", error=" + error + ", text='" + text + "'");
            if (emitType == SyscallProcessEmitType.Error) {
                Output.WriteLineProcess(emitType, process, thread, text + ": " + error.ToString());
            }
            else {
                Output.WriteLineProcess(emitType, process, thread, text);
            }

            writer.Write((int)Error.None);
        }

        public static void TimerCreate(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {

        }

        public static void Query(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {

        }
    }
}
