using Storm;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Threading.Tasks;

namespace Core
{
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

        public static Optional<Error> ProcessEmit(SyscallProcessEmitType emitType, Error error, string text)
        {
            var (reader, writer) = GetKernelSocket();
            writer.Write((int)SyscallNumber.ProcessEmit);
            writer.Write((int)emitType);
            writer.Write((int)error);
            SyscallHelpers.WriteText(writer, text);

            var kernelResult = (Error)reader.ReadInt32();
            if (kernelResult != 0) return new Optional<Error>(kernelResult); 
            return new Optional<Error>();
        }

        public static ErrorOr<Handle> ServiceCreate(ServiceDescription description)
        {
            var (reader, writer) = GetKernelSocket();
            writer.Write((int)SyscallNumber.ServiceCreate);
            SyscallHelpers.WriteText(writer, description.Protocol);
            SyscallHelpers.WriteText(writer, description.Vendor.HasValue() ? description.Vendor.Value() : null);
            SyscallHelpers.WriteText(writer, description.DeviceName.HasValue() ? description.DeviceName.Value() : null);
            SyscallHelpers.WriteGuid(writer, description.DeviceId.HasValue() ? description.DeviceId.Value() : null);

            var kernelResult = (Error)reader.ReadInt32();
            if (kernelResult != 0) return new ErrorOr<Handle>(kernelResult);
            var id = reader.ReadUInt64();
            return new ErrorOr<Handle>(new Handle(id));
        }
    }
}
