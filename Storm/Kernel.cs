﻿using Core;
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
                        // Service
                        case SyscallNumber.ServiceCreate:
                            SyscallHandlers.ServiceCreate(reader, writer, process);
                            break;

                        case SyscallNumber.ServiceDestroy:
                            SyscallHandlers.ServiceDestroy(reader, writer, process);
                            break;

                        case SyscallNumber.ServiceConnect:
                            SyscallHandlers.ServiceConnect(reader, writer, process);
                            break;

                        // Channel
                        case SyscallNumber.ChannelDestroy:
                            SyscallHandlers.ChannelDestroy(reader, writer, process);
                            break;

                        case SyscallNumber.ChannelSignal:
                            SyscallHandlers.ChannelSignal(reader, writer, process);
                            break;

                        // Event
                        case SyscallNumber.EventWait:
                            SyscallHandlers.EventWait(reader, writer, process);
                            break;

                        // Process
                        case SyscallNumber.ProcessDestroy:
                            running = false;
                            break;

                        case SyscallNumber.ProcessSetInfo:
                            SyscallHandlers.ProcessSetInfo(reader, writer, process);
                            break;

                        case SyscallNumber.ProcessEmit:
                            SyscallHandlers.ProcessEmit(reader, writer, process);
                            break;

                        // Unknown
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

            Handles.CleanupAfterProcess(process.PID);
            Services.CleanupProcess(process.PID);
            Output.WriteLine(SyscallProcessEmitType.Debug, process, "Application disconnected");
        }
    }
}