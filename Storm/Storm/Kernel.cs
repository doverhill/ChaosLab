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
                Output.WriteLineKernel(ProcessEmitType.Information, null, null, $"Starting {exePath} in {path} with delay {item.DelayMilliseconds}...");

                var startInfo = new ProcessStartInfo(exePath);
                startInfo.WorkingDirectory = path;
                var process = System.Diagnostics.Process.Start(startInfo);
                Process.CreateProcess((ulong)process.Id, null, item.Name, item.Capabilities, item.Grantables);
                Thread.Sleep(item.DelayMilliseconds);
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

            // sleep a little to make sure that the process has been registered
            Thread.Sleep(100);

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
                    running = SyscallHandlers.HandleSyscall(clientSocket, reader, writer, process, thread);
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