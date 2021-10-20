using Core;
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
            Output.WriteLine(SyscallProcessEmitType.Information, null, "Starting Storm kernel...");
            AcceptClients();
        }

        private void AcceptClients()
        {

            var ipEndpoint = new IPEndPoint(IPAddress.Parse("0.0.0.0"), 1337);
            var serverSocket = new Socket(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
            serverSocket.Bind(ipEndpoint);
            serverSocket.Listen();
            Output.WriteLine(SyscallProcessEmitType.Information, null, "Listening on " + ipEndpoint.ToString());

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
            var process = new Process(AllocatePID(), null);
            Output.WriteLine(SyscallProcessEmitType.Debug, process, "Application connected");

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
                            var error = (Error)reader.ReadInt32();
                            var text = SyscallHelpers.ReadText(reader);
                            SyscallHandlers.ProcessEmit(writer, process, emitType, error, text);
                            break;

                        case SyscallNumber.ProcessSetInfo:
                            var name = SyscallHelpers.ReadText(reader);
                            process.Name = name;
                            writer.Write((int)Error.None);
                            break;

                        case SyscallNumber.ProcessDestroy:
                            running = false;
                            break;

                        case SyscallNumber.ServiceCreate:
                            {
                                var protocol = SyscallHelpers.ReadText(reader);
                                var vendor = SyscallHelpers.ReadText(reader);
                                var deviceName = SyscallHelpers.ReadText(reader);
                                var deviceId = SyscallHelpers.ReadUuid(reader);
                                SyscallHandlers.ServiceCreate(writer, process, protocol, vendor, deviceName, deviceId);
                            }
                            break;

                        case SyscallNumber.ServiceConnect:
                            {
                                var protocol = SyscallHelpers.ReadText(reader);
                                var vendor = SyscallHelpers.ReadText(reader);
                                var deviceName = SyscallHelpers.ReadText(reader);
                                var deviceId = SyscallHelpers.ReadUuid(reader);
                                SyscallHandlers.ServiceConnect(writer, process, protocol, vendor, deviceName, deviceId);
                            }
                            break;

                        case SyscallNumber.EventWait:
                            var timeoutMilliseconds = reader.ReadInt32();
                            SyscallHandlers.EventWait(writer, process, timeoutMilliseconds);
                            break;

                        default:
                            Output.WriteLine(SyscallProcessEmitType.Error, process, "Unknown syscall: " + syscallNumber.ToString());
                            writer.Write((int)Error.NotImplemented);
                            break;
                    }
                }
            }
            catch (Exception e)
            {
                Output.WriteLine(SyscallProcessEmitType.Debug, process, "Application error: " + e.Message);
                clientSocket.Close();
            }

            Handles.CleanupProcess(process.PID);
            Services.CleanupProcess(process.PID);
            Output.WriteLine(SyscallProcessEmitType.Debug, process, "Application disconnected");
        }
    }
}