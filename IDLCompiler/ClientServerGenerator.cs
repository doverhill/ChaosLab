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

            source.AddLine("use alloc::boxed::Box;");
            source.AddLine("use library_chaos::{StormProcess, StormHandle};");
            source.AddBlank();

            var structBlock = source.AddBlock($"pub struct {protocolName.ToPascal()}{suffix}");
            structBlock.AddLine("channel_handle: StormHandle,");
            structBlock.AddLine("channel_address: *mut u8,");
            foreach (var call in from.Values)
            {
                structBlock.AddLine($"on_{call.Name}: Option<Box<dyn FnMut()>>,");
            }

            source.AddBlank();

            var implBlock = source.AddBlock($"impl {protocolName.ToPascal()}{suffix}");

            var createBlock = implBlock.AddBlock("pub fn create(process: &StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Option<StormHandle>");

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
