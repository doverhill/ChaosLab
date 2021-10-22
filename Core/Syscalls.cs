using System.IO;
using System.Net;
using System.Net.Sockets;
using Uuids;

namespace Core
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
        ProcessSetInfo = 9,
        ProcessEmit = 10,
        ProcessDestroy = 11,
        ThreadCreate = 12,
        ThreadDestroy = 13
    }

    public enum SyscallProcessEmitType
    {
        Error = 1,
        Warning = 2,
        Information = 3,
        Debug = 4
    }

    internal class Syscalls
    {
        private static object _lock = new object();
        private static Socket socket;
        private static NetworkStream socketStream;
        private static BinaryReader socketReader;
        private static BinaryWriter socketWriter;

        private static (BinaryReader Reader, BinaryWriter Writer) GetKernelSocket()
        {
            lock (_lock)
            {
                if (socket == null)
                {
                    var ipEndpoint = new IPEndPoint(IPAddress.Parse("127.0.0.1"), 1337);
                    socket = new Socket(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
                    socket.Connect(ipEndpoint);

                    socketStream = new NetworkStream(socket);
                    socketReader = new BinaryReader(socketStream);
                    socketWriter = new BinaryWriter(socketStream);
                }

                return (socketReader, socketWriter);
            }
        }

        public static void ProcessEnd()
        {
            var (reader, writer) = GetKernelSocket();
            writer.Write((int)SyscallNumber.ProcessDestroy);
        }

        public static Optional<Error> ProcessSetInfo(string processName)
        {
            var (reader, writer) = GetKernelSocket();
            writer.Write((int)SyscallNumber.ProcessSetInfo);
            SyscallHelpers.WriteText(writer, processName);

            var kernelResult = (Error)reader.ReadInt32();
            if (kernelResult != Error.None) return new Optional<Error>(kernelResult);
            return new Optional<Error>();
        }

        public static Optional<Error> ProcessEmit(SyscallProcessEmitType emitType, Error error, string text)
        {
            var (reader, writer) = GetKernelSocket();
            writer.Write((int)SyscallNumber.ProcessEmit);
            writer.Write((int)emitType);
            writer.Write((int)error);
            SyscallHelpers.WriteText(writer, text);

            var kernelResult = (Error)reader.ReadInt32();
            if (kernelResult != Error.None) return new Optional<Error>(kernelResult); 
            return new Optional<Error>();
        }

        public static ErrorOr<Handle> ServiceCreate(string protocolName, string vendorName, string deviceName, Uuid deviceId)
        {
            var (reader, writer) = GetKernelSocket();
            writer.Write((int)SyscallNumber.ServiceCreate);
            SyscallHelpers.WriteText(writer, protocolName);
            SyscallHelpers.WriteText(writer, vendorName);
            SyscallHelpers.WriteText(writer, deviceName);
            SyscallHelpers.WriteUuid(writer, deviceId);

            var kernelResult = (Error)reader.ReadInt32();
            if (kernelResult != Error.None) return new ErrorOr<Handle>(kernelResult);
            var id = reader.ReadUInt64();
            return new ErrorOr<Handle>(new Handle(id));
        }

        public static ErrorOr<Handle> ServiceConnect(string protocolName, string vendorName, string deviceName, Uuid? deviceId)
        {
            var (reader, writer) = GetKernelSocket();
            writer.Write((int)SyscallNumber.ServiceConnect);
            SyscallHelpers.WriteText(writer, protocolName);
            SyscallHelpers.WriteText(writer, vendorName);
            SyscallHelpers.WriteText(writer, deviceName);
            SyscallHelpers.WriteUuid(writer, deviceId);

            var kernelResult = (Error)reader.ReadInt32();
            if (kernelResult != Error.None) return new ErrorOr<Handle>(kernelResult);
            var id = reader.ReadUInt64();
            return new ErrorOr<Handle>(new Handle(id));
        }

        public static ErrorOr<Handle> EventWait()
        {
            return new ErrorOr<Handle>(Error.NotImplemented);
        }
    }
}
