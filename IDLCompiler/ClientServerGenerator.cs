using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal static class ClientServerGenerator
    {
        public static void GenerateSource(SourceGenerator source, IDL idl, Dictionary<string, IDLCall> from, Dictionary<string, IDLCall> to, string suffix)
        {
            var protocolName = CasedString.FromSnake(idl.Protocol.Name);

            source.AddLine("use library_chaos::{StormHandle};");

            var structBlock = source.AddBlock($"pub struct {protocolName.ToPascal()}{suffix}");
            structBlock.AddLine("channel_handle: StormHandle,");
            structBlock.AddLine("channel_address: *mut u8,");
            foreach (var call in from.Values)
            {
                structBlock.AddLine($"on_{call.Name}: Option<fn>,");
            }

            source.AddBlank();

            var implBlock = source.AddBlock($"impl {protocolName.ToPascal()}{suffix}");

            var createBlock = implBlock.AddBlock("pub fn create()");

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
