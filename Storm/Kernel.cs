using System.Collections.Concurrent;
using System.Net;
using System.Net.Sockets;
using System.Text;

namespace Storm
{
    public class Kernel
    {
        private object _lock = new object();
        private ulong nextPid = 1;

        public void Start()
        {
            Console.WriteLine("Starting Storm kernel...");

            AcceptClients();
        }

        private void AcceptClients()
        {

            var ipEndpoint = new IPEndPoint(IPAddress.Parse("0.0.0.0"), 1337);
            var serverSocket = new Socket(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
            serverSocket.Bind(ipEndpoint);
            serverSocket.Listen();
            Console.WriteLine("Listening on " + ipEndpoint.ToString());

            do
            {
                var clientSocket = serverSocket.Accept();
                var threadStart = new ParameterizedThreadStart(HandleClient);
                var thread = new Thread(threadStart);
                thread.Start(clientSocket);
            } while (true);
        }

        private ulong AllocatePID()
        {
            lock (_lock)
            {
                return nextPid++;
            }
        }

        private void HandleClient(object? socket)
        {
            var clientSocket = (Socket)socket;
            var pid = AllocatePID();
            Console.WriteLine("[" + pid + "] Application connected");

            using var clientStream = new NetworkStream(clientSocket);
            using var reader = new BinaryReader(clientStream);
            using var writer = new BinaryWriter(clientStream);

            try
            {
                bool running = true;
                while (running)
                {
                    var syscallNumber = (SyscallNumber)reader.ReadInt32();

                    switch (syscallNumber)
                    {
                        case SyscallNumber.ProcessEmit:
                            var emitType = (SyscallProcessEmitType)reader.ReadInt32();
                            var error = reader.ReadInt32();
                            var text = SyscallHelpers.ReadText(reader);
                            SyscallHandlers.ProcessEmit(writer, pid, emitType, error, text);
                            break;

                        case SyscallNumber.ProcessDestroy:
                            running = false;
                            break;

                        case SyscallNumber.ServiceCreate:
                            var protocol = SyscallHelpers.ReadText(reader);
                            var vendor = SyscallHelpers.ReadText(reader);
                            var deviceName = SyscallHelpers.ReadText(reader);
                            var deviceId = SyscallHelpers.ReadGuid(reader);
                            SyscallHandlers.ServiceCreate(writer, pid, protocol, vendor, deviceName, deviceId);
                            break;

                        default:
                            Console.WriteLine("Error: Unknown syscall: syscallNumber=" + syscallNumber.ToString() + ", PID=" + pid);
                            writer.Write((int)Error.NotImplemented);
                            break;
                    }
                }
            }
            catch (Exception e)
            {
            }

            Console.WriteLine("[" + pid + "] Application disconnected");
        }
    }
}