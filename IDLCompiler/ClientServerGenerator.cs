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
            source.AddLine($"use crate::channel::{channelName};");
            source.AddLine("use crate::from_client::*;");
            source.AddLine("use crate::from_server::*;");
            source.AddLine("use crate::MessageIds;");
            source.AddLine("use alloc::collections::BTreeMap;");
            source.AddBlank();

            var structBlock = source.AddBlock($"pub struct {structName}");
            structBlock.AddLine($"channels: BTreeMap<ChannelHandle, {channelName}>,");
            structBlock.AddLine("on_client_connected: Option<Box<dyn Fn(ChannelHandle)>>,");
            structBlock.AddLine("on_client_disconnected: Option<Box<dyn Fn(ChannelHandle)>>,");
            foreach (var call in from.Values)
            {
                structBlock.AddLine($"on_{call.Name}: Option<Box<dyn Fn(ChannelHandle)>>,");
            }

            source.AddBlank();

            var implBlock = source.AddBlock($"impl {structName}");

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
            var onConnect = implBlock.AddBlock("pub fn on_client_connected(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>)");
            onConnect.AddLine("self.on_client_connected = handler;");

            implBlock.AddBlank();
            onConnect = implBlock.AddBlock("pub fn on_client_disconnected(&mut self, handler: Option<Box<dyn Fn(ChannelHandle)>>)");
            onConnect.AddLine("self.on_client_disconnected = handler;");

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
                unsafeBlock.AddLine($"let address = channel.prepare_message(MessageIds::{parametersMessageName} as u64, {(call.Type == IDLCall.CallType.SingleEvent ? "true" : "false")});");
                if (parametersType != null)
                {
                    unsafeBlock.AddLine("let size = parameters.write_at(address);");
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
            source.AddLine($"use crate::channel::{channelName};");
            source.AddLine("use crate::from_client::*;");
            source.AddLine("use crate::from_server::*;");
            source.AddLine("use crate::MessageIds;");
            source.AddBlank();

            var structBlock = source.AddBlock($"pub struct {structName}");
            structBlock.AddLine($"channel: {channelName},");
            //structBlock.AddLine("channel_handle: ChannelHandle,");
            //structBlock.AddLine("channel_address: *mut u8,");
            foreach (var call in from.Values)
            {
                structBlock.AddLine($"on_{call.Name}: Option<Box<dyn Fn(ChannelHandle)>>,");
            }

            source.AddBlank();

            var implBlock = source.AddBlock($"impl {structName}");

            var connectBlock = implBlock.AddBlock("pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError>");
            connectBlock.AddLine($"let channel_handle = process.connect_to_service(\"{idl.Protocol.Name}\", None, None, None)?;");
            connectBlock.AddLine($$"""let channel = unsafe { {{channelName}}::new(process.get_channel_address(channel_handle).unwrap(), false) };""");
            var okBlock = connectBlock.AddBlock("Ok(Self");
            okBlock.Append = ")";
            //okBlock.AddLine("channel_handle: channel_handle,");
            //okBlock.AddLine("channel_address: process.get_channel_address(channel_handle).unwrap(),");
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
                    returns = " -> " + returnsType.Name;
                    parameters = ", process: &StormProcess";
                }
                if (parametersType != null)
                {
                    parameters += ", parameters: " + parametersType.Name;
                }

                var fnBlock = implBlock.AddBlock($"pub fn {call.Name}(&self{parameters}){returns}");
                var unsafeBlock = fnBlock.AddBlock("unsafe");
                unsafeBlock.AddLine($"let address = self.channel.prepare_message(MessageIds::{parametersMessageName} as u64, {(call.Type == IDLCall.CallType.SingleEvent ? "true" : "false")});");
                if (parametersType != null)
                {
                    unsafeBlock.AddLine("let size = parameters.write_at(address);");
                    unsafeBlock.AddLine("self.channel.commit_message(size);");
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

        private static void GenerateEventSetter(SourceGenerator.SourceBlock implBlock, IDLCall call)
        {
            var extraParameters = "";
            var fnBlock = implBlock.AddBlock($"pub fn on_{call.Name}(&mut self, handler: Option<Box<dyn Fn(ChannelHandle{extraParameters})>>)");
            fnBlock.AddLine($"self.on_{call.Name} = handler;");
            implBlock.AddBlank();

        }
    }
}
