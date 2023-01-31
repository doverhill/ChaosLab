using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Threading.Tasks.Dataflow;

namespace IDLCompiler
{
    internal static class ClientServerGenerator
    {
        public static void GenerateServer(SourceGenerator source, IDL idl, Dictionary<string, IDLCall> from, Dictionary<string, IDLCall> to)
        {
            var protocolName = CasedString.FromSnake(idl.Protocol.Name);
            var structName = protocolName.ToPascal() + "Server";
            var channelName = protocolName.ToPascal() + "Channel";

            source.AddLine("use alloc::boxed::Box;");
            source.AddLine("use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError};");
            source.AddLine("use uuid::Uuid;");
            source.AddLine($$"""use crate::channel::{{{channelName}}, ChannelMessageHeader};""");
            source.AddLine("use crate::from_client::*;");
            source.AddLine("use crate::from_server::*;");
            source.AddLine("use crate::MessageIds;");
            source.AddLine("use alloc::collections::BTreeMap;");
            source.AddBlank();

            var structBlock = source.AddBlock($"pub struct {structName}<'a>");
            structBlock.AddLine($"channels: BTreeMap<ChannelHandle, {channelName}>,");
            structBlock.AddLine("on_client_connected: Option<Box<dyn Fn(ChannelHandle) + 'a>>,");
            structBlock.AddLine("on_client_disconnected: Option<Box<dyn Fn(ChannelHandle) + 'a>>,");
            foreach (var call in from.Values)
            {
                structBlock.AddLine($"on_{call.Name}: Option<Box<dyn Fn(ChannelHandle) + 'a>>,");
            }

            source.AddBlank();

            var implBlock = source.AddBlock($"impl<'a> {structName}<'a>");

            var createBlock = implBlock.AddBlock("pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError>");
            createBlock.AddLine($"let service_handle = process.create_service(\"{idl.Protocol.Name}\", vendor_name, device_name, device_id)?;");
            var okBlock = createBlock.AddBlock("Ok(Self");
            okBlock.Append = ")";
            okBlock.AddLine("channels: BTreeMap::new(),");
            okBlock.AddLine("on_client_connected: None,");
            okBlock.AddLine("on_client_disconnected: None,");
            foreach (var call in from.Values)
            {
                okBlock.AddLine($"on_{call.Name}: None,");
            }

            implBlock.AddBlank();
            var onConnect = implBlock.AddBlock("pub fn on_client_connected(&mut self, handler: impl Fn(ChannelHandle) + 'a)");
            onConnect.AddLine("self.on_client_connected = Some(Box::new(handler));");
            implBlock.AddBlank();

            onConnect = implBlock.AddBlock("pub fn clear_on_client_connected(&mut self)");
            onConnect.AddLine("self.on_client_connected = None;");
            implBlock.AddBlank();

            onConnect = implBlock.AddBlock("pub fn on_client_disconnected(&mut self, handler: impl Fn(ChannelHandle) + 'a)");
            onConnect.AddLine("self.on_client_disconnected = Some(Box::new(handler));");
            implBlock.AddBlank();

            onConnect = implBlock.AddBlock("pub fn clear_on_client_disconnected(&mut self)");
            onConnect.AddLine("self.on_client_disconnected = None;");
            implBlock.AddBlank();

            foreach (var call in to.Values)
            {
                var (parametersType, parametersMessageName) = call.ToParametersType();
                var (returnsType, returnsMessageName) = call.ToReturnsType(true);

                string parameters = "";
                if (parametersType != null)
                {
                    parameters = ", parameters: " + parametersType.Name;
                }

                string returns = "";
                if (returnsType != null)
                {
                    returns = " -> " + returnsType.Name;
                }

                var fnBlock = implBlock.AddBlock($"pub fn {call.Name}(&self, channel_handle: ChannelHandle{parameters}){returns}");
                var ifBlock = fnBlock.AddBlock("if let Some(channel) = self.channels.get(&channel_handle)");
                var unsafeBlock = ifBlock.AddBlock("unsafe");
                unsafeBlock.AddLine($"let message = channel.prepare_message(MessageIds::{parametersMessageName} as u64, {(call.Type == IDLCall.CallType.SingleEvent ? "true" : "false")});");
                if (parametersType != null)
                {
                    unsafeBlock.AddLine("let payload = ChannelMessageHeader::get_payload_address(message);");
                    unsafeBlock.AddLine("let size = parameters.write_at(payload);");
                    unsafeBlock.AddLine("channel.commit_message(size);");
                }
                else
                {
                    unsafeBlock.AddLine("self.channel.commit_message(0);");
                }

                implBlock.AddBlank();
            }

            foreach (var call in from.Values)
            {
                GenerateEventSetter(implBlock, call);
            }

        }

        public static void GenerateClient(SourceGenerator source, IDL idl, Dictionary<string, IDLCall> from, Dictionary<string, IDLCall> to)
        {
            var protocolName = CasedString.FromSnake(idl.Protocol.Name);
            var structName = protocolName.ToPascal() + "Client";
            var channelName = protocolName.ToPascal() + "Channel";

            source.AddLine("use alloc::boxed::Box;");
            source.AddLine("use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError};");
            source.AddLine("use uuid::Uuid;");
            source.AddLine($$"""use crate::channel::{{{channelName}}, ChannelMessageHeader, FromChannel};""");
            source.AddLine("use crate::from_client::*;");
            source.AddLine("use crate::from_server::*;");
            source.AddLine("use crate::MessageIds;");
            source.AddBlank();

            var structBlock = source.AddBlock($"pub struct {structName}<'a>");
            structBlock.AddLine("channel_handle: ChannelHandle,");
            structBlock.AddLine($"channel: {channelName},");
            //structBlock.AddLine("channel_handle: ChannelHandle,");
            //structBlock.AddLine("channel_address: *mut u8,");
            foreach (var call in from.Values)
            {
                structBlock.AddLine($"on_{call.Name}: Option<Box<dyn Fn(ChannelHandle) + 'a>>,");
            }

            source.AddBlank();

            var implBlock = source.AddBlock($"impl<'a> {structName}<'a>");

            var connectBlock = implBlock.AddBlock("pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError>");
            connectBlock.AddLine($"let channel_handle = process.connect_to_service(\"{idl.Protocol.Name}\", None, None, None)?;");
            connectBlock.AddLine($$"""let channel = unsafe { {{channelName}}::new(process.get_channel_address(channel_handle).unwrap(), false) };""");
            var okBlock = connectBlock.AddBlock("Ok(Self");
            okBlock.Append = ")";
            //okBlock.AddLine("channel_handle: channel_handle,");
            //okBlock.AddLine("channel_address: process.get_channel_address(channel_handle).unwrap(),");
            okBlock.AddLine("channel_handle: channel_handle,");
            okBlock.AddLine("channel: channel,");
            foreach (var call in from.Values)
            {
                okBlock.AddLine($"on_{call.Name}: None,");
            }

            implBlock.AddBlank();

            foreach (var call in to.Values)
            {
                var (parametersType, parametersMessageName) = call.ToParametersType();
                var (returnsType, returnsMessageName) = call.ToReturnsType(false);

                string parameters = "";
                string returns = "";
                if (returnsType != null)
                {
                    returns = $" -> Result<FromChannel<&{returnsType.Name}>, StormError>";
                    parameters = ", process: &StormProcess";
                }
                if (parametersType != null)
                {
                    parameters += ", parameters: &" + parametersType.Name;
                }

                var fnBlock = implBlock.AddBlock($"pub fn {call.Name}(&self{parameters}){returns}");
                var unsafeBlock = fnBlock.AddBlock("unsafe");
                unsafeBlock.AddLine($"let message = self.channel.prepare_message(MessageIds::{parametersMessageName} as u64, {(call.Type == IDLCall.CallType.SingleEvent ? "true" : "false")});");
                if (parametersType != null)
                {
                    unsafeBlock.AddLine("let payload = ChannelMessageHeader::get_payload_address(message);");
                    unsafeBlock.AddLine("let size = parameters.write_at(payload);");
                    unsafeBlock.AddLine("self.channel.commit_message(size);");
                }
                else
                {
                    unsafeBlock.AddLine("self.channel.commit_message(0);");
                }

                if (returnsType != null)
                {
                    fnBlock.AddBlank();
                    fnBlock.AddLine($"process.wait_for_channel_message(self.channel_handle, MessageIds::{returnsMessageName} as u64, 1000)?;");
                    fnBlock.AddBlank();

                    unsafeBlock = fnBlock.AddBlock("unsafe");
                    var ifBlock = unsafeBlock.AddBlock($"if let Some(message) = self.channel.find_specific_message(MessageIds::{returnsMessageName} as u64)");
                    ifBlock.AddLine("let payload = ChannelMessageHeader::get_payload_address(message);");
                    ifBlock.AddLine($"{returnsType.Name}::reconstruct_at_inline(payload);");
                    ifBlock.AddLine($"let payload = payload as *mut {returnsType.Name};");
                    ifBlock.AddLine($"Ok(FromChannel::new(payload.as_ref().unwrap()))");
                    var elseBlock = unsafeBlock.AddBlock("else");
                    elseBlock.AddLine("Err(StormError::NotFound)");
                }

                implBlock.AddBlank();
            }

            foreach (var call in from.Values)
            {
                GenerateEventSetter(implBlock, call);
            }

        }

        private static void GenerateEventSetter(SourceGenerator.SourceBlock implBlock, IDLCall call)
        {
            var extraParameters = "";
            var fnBlock = implBlock.AddBlock($"pub fn on_{call.Name}(&mut self, handler: impl Fn(ChannelHandle{extraParameters}) + 'a)");
            fnBlock.AddLine($"self.on_{call.Name} = Some(Box::new(handler));");
            implBlock.AddBlank();

            fnBlock = implBlock.AddBlock($"pub fn clear_on_{call.Name}(&mut self)");
            fnBlock.AddLine($"self.on_{call.Name} = None;");
            implBlock.AddBlank();
        }
    }
}
