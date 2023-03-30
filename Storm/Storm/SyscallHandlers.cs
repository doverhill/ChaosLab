using System.IO;
using System.Net.Sockets;

namespace Storm {
    internal static class SyscallHandlers {
        public static void ServiceCreate(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var protocol = SyscallHelpers.ReadText(reader);
            var deviceId = SyscallHelpers.ReadUuid(reader);

            if (protocol == null || !deviceId.HasValue) {
                writer.Write((int)ErrorCode.Malformed);
                return;
            }

            if (process.HasStormCapability("ServiceCreate", protocol)) {
                var owner = process.TrustChain;
                Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ServiceCreate: protocol='" + protocol + "', owner='" + owner + "', deviceId=" + deviceId);
                var handle = Services.Create(process, protocol, owner, deviceId.Value);

                if (handle.IsError) {
                    writer.Write((int)handle.ErrorCode);
                }
                else {
                    writer.Write((int)ErrorCode.None);
                    writer.Write(handle.Value);
                }
            }
            else {
                writer.Write((int)ErrorCode.PermissionDenied);
            }
        }

        //public static void ServiceDestroy(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
        //    var serviceHandleId = reader.ReadUInt64();

        //    Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ServiceDestroy: handleId=" + serviceHandleId);
        //    var success = Services.Destroy(process.ProcessId, serviceHandleId);

        //    if (success) {
        //        writer.Write((int)Error.None);
        //    }
        //    else {
        //        writer.Write((int)Error.PermissionDenied);
        //    }
        //}

        public static void ServiceSubscribe(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var protocol = SyscallHelpers.ReadText(reader);
            var owner = SyscallHelpers.ReadText(reader);
            var deviceId = SyscallHelpers.ReadUuid(reader);

            if (protocol == null) {
                writer.Write((int)ErrorCode.Malformed);
                return;
            }

            if (process.HasStormCapability("ServiceConnect", protocol)) {
                Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ServiceSubscribe: protocol='" + protocol + "', owner='" + (owner ?? "*") + "', deviceId=" + (deviceId?.ToString() ?? "*"));
                var handle = Services.CreateSubscription(process, protocol, owner, deviceId);

                if (handle.IsError) {
                    writer.Write((int)handle.ErrorCode);
                }
                else {
                    writer.Write((int)ErrorCode.None);
                    writer.Write(handle.Value);
                }
            }
            else {
                writer.Write((int)ErrorCode.PermissionDenied);
            }
        }

        //public static void ChannelDestroy(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
        //    var channelHandleId = reader.ReadUInt64();

        //    Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ChannelDestroy: handleId=" + channelHandleId);
        //    var success = Handles.Destroy(process.ProcessId, channelHandleId);

        //    if (success) {
        //        writer.Write((int)ErrorCode.None);
        //    }
        //    else {
        //        writer.Write((int)ErrorCode.PermissionDenied);
        //    }
        //}

        public static void ChannelSignal(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var channelHandleId = reader.ReadUInt64();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ChannelSignal: handleId=" + channelHandleId);
            var handle = Handles.GetHandle(process.ProcessId, channelHandleId, Handle.Type.Channel);

            if (handle.HasValue) {
                var targetProcessId = handle.Value.GetOtherProcessId(process.ProcessId);
                var targetProcess = Process.FindProcess(targetProcessId);
                targetProcess.SetChannelSignalled(channelHandleId);
                targetProcess.PostProcessFlagsEvent();

                writer.Write((int)ErrorCode.None);
            }
            else {
                writer.Write((int)ErrorCode.PermissionDenied);
            }
        }

        public static void EventWait(Socket socket, BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var handle = SyscallHelpers.ReadOptionalU64(reader);
            var action = (HandleAction?)SyscallHelpers.ReadOptionalI32(reader);
            var timeoutMilliseconds = reader.ReadInt32();

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL EventWait: handle=" + (handle?.ToString() ?? "*") + ", action=" + (action?.ToString() ?? "*") + ", timeout=" + timeoutMilliseconds);
            var e = Events.Wait(socket, process, handle, action, timeoutMilliseconds);

            writer.Write((int)e.Error);
            if (e.Error == ErrorCode.None) {
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
            var path = SyscallHelpers.ReadText(reader);
            var numberOfCapabilities = reader.ReadInt32();
            var capabilites = new List<string>();
            for (var index = 0; index < numberOfCapabilities; index++) {
                capabilites.Add(SyscallHelpers.ReadText(reader));
            }
            var numberOfGrantables = reader.ReadInt32();
            var grantables = new List<string>();
            for (var index = 0; index < numberOfGrantables; index++) {
                grantables.Add(SyscallHelpers.ReadText(reader));
            }
            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ProcessCreate: path=" + path + ", capabilities=" + string.Join(", ", capabilites) + ", grantables=" + string.Join(", ", grantables));

            var name = Path.GetFileName(path);

            // FIXME
            // start process
            ulong processId = 324233;

            var result = Process.CreateProcess(processId, process, name, capabilites, grantables);
        }

        public static void ProcessEmit(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var emitType = (SyscallProcessEmitType)reader.ReadInt32();
            var error = (ErrorCode)reader.ReadInt32();
            var text = SyscallHelpers.ReadText(reader);

            Output.WriteLineKernel(SyscallProcessEmitType.Debug, process, thread, "SYSCALL ProcessEmit: type=" + emitType.ToString() + ", error=" + error + ", text='" + text + "'");
            if (emitType == SyscallProcessEmitType.Error) {
                Output.WriteLineProcess(emitType, process, thread, text + ": " + error.ToString());
            }
            else {
                Output.WriteLineProcess(emitType, process, thread, text);
            }

            writer.Write((int)ErrorCode.None);
        }

        public static void ProcessReduceCapabilities(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            // FIXME
            writer.Write((int)ErrorCode.None);
        }

        public static void TimerCreate(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            // FIXME

        }

        public static void Query(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            // FIXME

        }

        public static void HandleDestroy(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            // FIXME
        }
    }
}
