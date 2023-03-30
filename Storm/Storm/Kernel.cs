using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Threading;

namespace Storm {
    public class Kernel {
        public void Start(List<StartupCommand> startupList) {
            Output.WriteLineKernel(ProcessEmitType.Information, null, null, "Starting Storm...");
            Output.WriteLineKernel(ProcessEmitType.Debug, null, null, $"{startupList.Count} startup commands");

            var threadStart = new ParameterizedThreadStart(HandleStartup);
            var startupThread = new Thread(threadStart);
            startupThread.Start(startupList);

            AcceptClients();
        }

        private void HandleStartup(object? list) {
            var startupList = (List<StartupCommand>)list;

            Thread.Sleep(200);

            foreach (var item in startupList) {
                var path = Path.Combine(Environment.CurrentDirectory, item.Path);
                var exePath = Path.Combine(path, item.Executable);
                Output.WriteLineKernel(ProcessEmitType.Information, null, null, $"Starting {exePath} in {path} with delay {item.DelayMs}...");

                var startInfo = new ProcessStartInfo(exePath);
                startInfo.WorkingDirectory = path;
                System.Diagnostics.Process.Start(startInfo);
                Thread.Sleep(item.DelayMs);
            }
        }

        private void AcceptClients() {
            var ipEndpoint = new IPEndPoint(IPAddress.Parse("0.0.0.0"), 1337);
            var serverSocket = new Socket(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
            serverSocket.Bind(ipEndpoint);
            serverSocket.Listen();
            Output.WriteLineKernel(ProcessEmitType.Information, null, null, "Storm started. Listening on " + ipEndpoint.ToString());

            do {
                var clientSocket = serverSocket.Accept();
                var threadStart = new ParameterizedThreadStart(HandleClient);
                var thread = new Thread(threadStart);
                thread.Start(clientSocket);
            } while (true);
        }

        private void HandleClient(object? socket) {
            var clientSocket = (Socket)socket;

            using var clientStream = new NetworkStream(clientSocket);
            using var reader = new BinaryReader(clientStream);
            using var writer = new BinaryWriter(clientStream);

            var processId = reader.ReadUInt64();
            var threadId = reader.ReadUInt64();

            var processResult = Process.GetProcess(processId, threadId);
            if (processResult.IsError) {
                Output.WriteLineKernel(ProcessEmitType.Warning, null, null, "Ignoring connection from unknown process");
                clientSocket.Close();
                return;
            }
            var (process, thread) = processResult.Value;

            Output.WriteLineKernel(ProcessEmitType.Debug, process, thread, "New connection");

            try {
                bool running = true;
                while (running) {
                    var syscallNumber = (SyscallNumber)reader.ReadInt32();

                    switch (syscallNumber) {
                        // Service
                        case SyscallNumber.ServiceCreate:
                            SyscallHandlers.ServiceCreate(reader, writer, process, thread);
                            break;

                        case SyscallNumber.ServiceSubscribe:
                            SyscallHandlers.ServiceSubscribe(reader, writer, process, thread);
                            break;

                        // Channel
                        case SyscallNumber.ChannelSignal:
                            SyscallHandlers.ChannelSignal(reader, writer, process, thread);
                            break;

                        // Event
                        case SyscallNumber.EventWait:
                            SyscallHandlers.EventWait(clientSocket, reader, writer, process, thread);
                            break;

                        // Process
                        case SyscallNumber.ProcessCreate:
                            SyscallHandlers.ProcessCreate(reader, writer, process, thread);
                            break;

                        case SyscallNumber.ProcessEmit:
                            SyscallHandlers.ProcessEmit(reader, writer, process, thread);
                            break;

                        case SyscallNumber.ProcessReduceCapabilities:
                            SyscallHandlers.ProcessReduceCapabilities(reader, writer, process, thread);
                            break;

                        // Timer
                        case SyscallNumber.TimerCreate:
                            SyscallHandlers.TimerCreate(reader, writer, process, thread);
                            break;

                        // Query
                        case SyscallNumber.Query:
                            SyscallHandlers.Query(reader, writer, process, thread);
                            break;

                        // Handle
                        case SyscallNumber.HandleDestroy:
                            SyscallHandlers.HandleDestroy(reader, writer, process, thread);
                            break;

                        // Unknown
                        default:
                            Output.WriteLineKernel(ProcessEmitType.Error, process, thread, "Unknown syscall: " + syscallNumber.ToString());
                            writer.Write((int)ErrorCode.NotImplemented);
                            break;
                    }
                }
            }
            catch (Exception e) {
                Output.WriteLineKernel(ProcessEmitType.Information, process, thread, "Application error: " + e.Message);
                clientSocket.Close();
            }

            var processDeleted = Process.RemoveThread(process, thread);
            if (processDeleted) {
                HandleCollection.CleanupAfterProcess(process);
                //Services.Cleanup(process);
                Output.WriteLineKernel(ProcessEmitType.Information, process, thread, "Process exit");
            }
            else {
                Output.WriteLineKernel(ProcessEmitType.Information, process, thread, "Thread exit");
            }
        }
    }
}