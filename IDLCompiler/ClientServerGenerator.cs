using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal static class ClientServerGenerator
    {
        public enum ClientServerSide
        {
            Client,
            Server
        }

        public static void GenerateSource(SourceGenerator source, IDL idl, Dictionary<string, IDLCall> from, Dictionary<string, IDLCall> to, ClientServerSide side)
        {
            var protocolName = CasedString.FromSnake(idl.Protocol.Name);
            var suffix = side == ClientServerSide.Server ? "Server" : "Client";
            var structName = protocolName + suffix;

            source.AddLine("use alloc::boxed::Box;");
            source.AddLine("use library_chaos::{StormProcess, StormHandle, StormError, syscalls};");
            source.AddLine("use uuid::Uuid;");
            source.AddBlank();

            var structBlock = source.AddBlock($"pub struct {structName}");
            structBlock.AddLine("channel_handle: StormHandle,");
            structBlock.AddLine("channel_address: *mut u8,");
            foreach (var call in from.Values)
            {
                structBlock.AddLine($"on_{call.Name}: Option<Box<dyn FnMut()>>,");
            }

            source.AddBlank();

            var implBlock = source.AddBlock($"impl {structName}");

            if (side == ClientServerSide.Server)
            {
                var createBlock = implBlock.AddBlock("pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Option<Self>");
            }
            else
            {
                var connectBlock = implBlock.AddBlock("pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError>");
                var matchBlock = connectBlock.AddBlock($"match syscalls::connect(\"{idl.Protocol.Name}\", None, None, None, 4096)");
                var okBlock = connectBlock.AddBlock("Ok(service_handle) =>");
                okBlock.AddLine("Ok(Self { service_handle");
            }
            implBlock.AddBlank();

            foreach (var call in to.Values)
            {
                var fnBlock = implBlock.AddBlock($"pub fn {call.Name}()");

                implBlock.AddBlank();
            }

            foreach (var call in from.Values)
            {
                var fnBlock = implBlock.AddBlock($"pub fn on_{call.Name}()");

                implBlock.AddBlank();
            }

        }
    }
}
