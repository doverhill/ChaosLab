using System.Collections.Generic;
using System.IO;
using System.Net.Sockets;

namespace Storm {
    internal static class SyscallHandlers {
        public static bool HandleSyscall(Socket socket, BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var syscallNumber = (SyscallNumber)reader.ReadInt32();

            switch (syscallNumber) {
                // Service
                case SyscallNumber.ServiceCreate:
                    ServiceCreate(reader, writer, process, thread);
                    break;

                case SyscallNumber.ServiceSubscribe:
                    ServiceSubscribe(reader, writer, process, thread);
                    break;

                // Channel
                case SyscallNumber.ChannelSignal:
                    ChannelSignal(reader, writer, process, thread);
                    break;

                // Event
                case SyscallNumber.EventWait:
                    EventWait(socket, reader, writer, process, thread);
                    break;

                // Process
                case SyscallNumber.ProcessCreate:
                    ProcessCreate(reader, writer, process, thread);
                    break;

                case SyscallNumber.ProcessEmit:
                    ProcessEmit(reader, writer, process, thread);
                    break;

                case SyscallNumber.ProcessReduceCapabilities:
                    ProcessReduceCapabilities(reader, writer, process, thread);
                    break;

                // Timer
                case SyscallNumber.TimerCreate:
                    TimerCreate(reader, writer, process, thread);
                    break;

                // Query
                case SyscallNumber.Query:
                    Query(reader, writer, process, thread);
                    break;

                // Handle
                case SyscallNumber.HandleDestroy:
                    HandleDestroy(reader, writer, process, thread);
                    break;

                // Unknown
                default:
                    Output.WriteLineKernel(ProcessEmitType.Error, process, thread, "Unknown syscall: " + syscallNumber.ToString());
                    writer.Write((int)ErrorCode.NotImplemented);
                    break;
            }

            return true;
        }

        public static void ServiceCreate(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var protocol = SyscallHelpers.ReadText(reader);
            var deviceId = SyscallHelpers.ReadUuid(reader);

            if (protocol == null || !deviceId.HasValue) {
                writer.Write((int)ErrorCode.Malformed);
                return;
            }

            if (process.HasStormCapability("ServiceCreate", protocol)) {
                var owner = process.TrustChain;
                Output.WriteLineKernel(ProcessEmitType.Debug, process, thread, "SYSCALL ServiceCreate: protocol='" + protocol + "', owner='" + owner + "', deviceId=" + deviceId);
                var handle = ServiceCollection.Create(process, protocol, owner, deviceId.Value);

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
                Output.WriteLineKernel(ProcessEmitType.Debug, process, thread, "SYSCALL ServiceSubscribe: protocol='" + protocol + "', owner='" + (owner ?? "*") + "', deviceId=" + (deviceId?.ToString() ?? "*"));
                var handle = ServiceCollection.CreateSubscription(process, protocol, owner, deviceId);

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

            Output.WriteLineKernel(ProcessEmitType.Debug, process, thread, "SYSCALL ChannelSignal: handleId=" + channelHandleId);
            var handle = HandleCollection.GetHandle(process.ProcessId, channelHandleId, Handle.HandleType.Channel);

            if (handle.HasValue) {
                var targetProcessId = handle.Value.GetOtherProcessId(process.ProcessId);
                var targetProcess = Process.FindProcess(targetProcessId);
                targetProcess.SetChannelSignalled(channelHandleId);
                writer.Write((int)ErrorCode.None);
            }
            else {
                writer.Write((int)ErrorCode.PermissionDenied);
            }
        }

        public static void EventWait(Socket socket, BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var handle = SyscallHelpers.ReadOptionalU64(reader);
            var eventType = (Event.EventType?)SyscallHelpers.ReadOptionalI32(reader);
            var timeoutMilliseconds = reader.ReadInt32();

            Output.WriteLineKernel(ProcessEmitType.Debug, process, thread, "SYSCALL EventWait: handle=" + (handle?.ToString() ?? "*") + ", type=" + (eventType?.ToString() ?? "*") + ", timeout=" + timeoutMilliseconds);
            if (process.WaitEvent(socket, handle, eventType, out var stormEvent, timeoutMilliseconds)) {
                writer.Write((int)ErrorCode.None);
                writer.Write(stormEvent.TargetHandleId);
                writer.Write(stormEvent.AdditionalHandleId);
                writer.Write((int)stormEvent.Type);
            }
            else {
                writer.Write((int)ErrorCode.Timeout);
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
            Output.WriteLineKernel(ProcessEmitType.Debug, process, thread, "SYSCALL ProcessCreate: path=" + path + ", capabilities=" + string.Join(", ", capabilites) + ", grantables=" + string.Join(", ", grantables));

            var name = Path.GetFileName(path);

            // FIXME
            // start process
            ulong processId = 324233;

            var result = Process.CreateProcess(processId, process, name, capabilites, grantables);
        }

        public static void ProcessEmit(BinaryReader reader, BinaryWriter writer, Process process, Process.Thread thread) {
            var emitType = (ProcessEmitType)reader.ReadInt32();
            var error = (ErrorCode)reader.ReadInt32();
            var text = SyscallHelpers.ReadText(reader);

            Output.WriteLineKernel(ProcessEmitType.Debug, process, thread, "SYSCALL ProcessEmit: type=" + emitType.ToString() + ", error=" + error + ", text='" + text + "'");
            if (emitType == ProcessEmitType.Error) {
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
