using System.Collections.Generic;
using System.Runtime.CompilerServices;

namespace IDLCompiler {
    internal static class ClientServerGenerator
    {
        public static void GenerateServer(SourceGenerator source, IDL idl, Dictionary<string, IDLCall> from, Dictionary<string, IDLCall> to, int initialFromSize, int intialToSize)
        {
            var protocolName = CasedString.FromSnake(idl.Protocol.Name);
            var structName = protocolName.ToPascal() + "Server";
            var channelName = protocolName.ToPascal() + "Channel";

            source.AddLine("use alloc::boxed::Box;");
            source.AddLine("use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, StormEvent};");
            source.AddLine("use uuid::Uuid;");
            source.AddLine($$"""use crate::channel::{{{channelName}}, ChannelMessageHeader};""");
            source.AddLine("use crate::from_client::*;");
            source.AddLine("use crate::from_server::*;");
            source.AddLine("use crate::message_ids::*;");
            source.AddLine("use alloc::collections::BTreeMap;");
            source.AddLine("use alloc::vec::Vec;");
            source.AddBlank();

            var requestEnumBlock = source.AddBlock($"pub enum {structName}Request<'a>");
            foreach (var call in from.Values)
            {
                var callName = CasedString.FromSnake(call.Name);
                var (parametersType, _) = call.ToParametersType();
                var parameter = parametersType != null ? $"(&'a {parametersType.Name})" : "";
                requestEnumBlock.AddLine($"{callName.ToPascal()}{parameter},");
            }
            source.AddBlank();

            var observerTrait = source.AddBlock($"pub trait {structName}Observer");
            observerTrait.AddLine($"fn handle_{idl.Protocol.Name}_client_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);");
            observerTrait.AddLine($"fn handle_{idl.Protocol.Name}_client_disconnected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);");
            observerTrait.AddLine($"fn handle_{idl.Protocol.Name}_request(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, request: {structName}Request);");
            source.AddBlank();

            var structBlock = source.AddBlock($"pub struct {structName}");
            structBlock.AddLine("service_handle: ServiceHandle,");
            structBlock.AddLine($"channels: BTreeMap<ChannelHandle, {channelName}>,");

            source.AddBlank();

            var implBlock = source.AddBlock($"impl {structName}");

            var createBlock = implBlock.AddBlock("pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError>");
            createBlock.AddLine($"let service_handle = process.create_service(\"{idl.Protocol.Name}\", vendor_name, device_name, device_id)?;");
            var okBlock = createBlock.AddBlock("Ok(Self");
            okBlock.Append = ")";
            okBlock.AddLine("service_handle: service_handle,");
            okBlock.AddLine("channels: BTreeMap::new(),");
            implBlock.AddBlank();

            var processBlock = implBlock.AddBlock($"pub fn process_event(&mut self, process: &mut StormProcess, event: &StormEvent, observer: &mut impl {structName}Observer)");
            var matchBlock = processBlock.AddBlock("match event");
            var connectBlock = matchBlock.AddBlock("StormEvent::ServiceConnected(service_handle, channel_handle) =>");
            
            //connectBlock.AddLine("println!(\"{:?} == {:?}?\", *service_handle, self.service_handle);");
            var ifBlock = connectBlock.AddBlock("if *service_handle == self.service_handle");
            ifBlock.AddLine($"println!(\"{structName}: client connected\");");
            ifBlock.AddLine($"process.initialize_channel(*channel_handle, {initialFromSize});");
            ifBlock.AddLine($"let channel = {channelName}::new(process.get_channel_address(*channel_handle, 0).unwrap(), process.get_channel_address(*channel_handle, 1).unwrap(), true);");
            ifBlock.AddLine("self.channels.insert(*channel_handle, channel);");
            ifBlock.AddLine($"observer.handle_{idl.Protocol.Name}_client_connected(*service_handle, *channel_handle);");

            var messageBlock = matchBlock.AddBlock("StormEvent::ChannelSignalled(channel_handle) =>");
            ifBlock = messageBlock.AddBlock("if let Some(channel) = self.channels.get(&channel_handle)");
            //ifBlock.AddLine($"println!(\"{structName}: client request\");");
            var whileBlock = ifBlock.AddBlock("while let Some(message) = channel.find_message()");
            //whileBlock.AddLine("println!(\"found channel message\");");
            var unsafeBlock = whileBlock.AddBlock("unsafe");
            var innerMatchBlock = unsafeBlock.AddBlock("match (*message).message_id");
            foreach (var call in from.Values)
            {
                var callName = CasedString.FromSnake(call.Name);
                var (parametersType, parametersMessageName) = call.ToParametersType();
                var arm = innerMatchBlock.AddBlock($"{parametersMessageName} => ");
                //arm.AddLine($$"""println!("got {{parametersMessageName}} message");""");
                if (parametersType != null) {
                    arm.AddLine("let address = ChannelMessageHeader::get_payload_address(message);");
                    arm.AddLine($"{parametersType.Name}::reconstruct_at_inline(address);");
                    arm.AddLine($"let parameters = address as *const {parametersType.Name};");
                    arm.AddLine($"let request = {structName}Request::{callName.ToPascal()}(parameters.as_ref().unwrap());");
                    arm.AddLine($"observer.handle_{idl.Protocol.Name}_request(self.service_handle, *channel_handle, request);");
                }
                else {
                    arm.AddLine($"observer.handle_{idl.Protocol.Name}_request(self.service_handle, *channel_handle, {structName}Request::{callName.ToPascal()});");
                }
                arm.AddLine("channel.unlink_message(message, false);");
            }
            innerMatchBlock.AddLine("_ => {}");

            var destroyBlock = matchBlock.AddBlock("StormEvent::ChannelDestroyed(channel_handle) =>");
            ifBlock = destroyBlock.AddBlock("if let Some(_) = self.channels.get(&channel_handle)");
            ifBlock.AddLine($"println!(\"{structName}: client disconnected\");");
            ifBlock.AddLine($"observer.handle_{idl.Protocol.Name}_client_disconnected(self.service_handle, *channel_handle);");
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

                var fnBlock = implBlock.AddBlock($"pub fn {call.Name}(&mut self, channel_handle: ChannelHandle{parameters}){returns}");
                //fnBlock.AddLine($"println!(\"{structName}::{call.Name}\");");
                ifBlock = fnBlock.AddBlock("if let Some(channel) = self.channels.get_mut(&channel_handle)");
                //ifBlock.AddLine("println!(\"found channel\");");
                ifBlock.AddLine($"let (_, message) = channel.prepare_message({parametersMessageName}, {(call.Type == IDLCall.CallType.SingleEvent ? "true" : "false")});");
                if (parametersType != null)
                {
                    ifBlock.AddLine("let payload = ChannelMessageHeader::get_payload_address(message);");
                    ifBlock.AddLine("let size = unsafe { parameters.write_at(payload) };");
                    ifBlock.AddLine("channel.commit_message(size);");
                    ifBlock.AddLine($"StormProcess::signal_channel(channel_handle);");
                }
                else
                {
                    ifBlock.AddLine("self.channel.commit_message(0);");
                }

                implBlock.AddBlank();
            }
        }

        public static void GenerateClient(SourceGenerator source, IDL idl, Dictionary<string, IDLCall> from, Dictionary<string, IDLCall> to, int initialFromSize, int intialToSize)
        {
            var protocolName = CasedString.FromSnake(idl.Protocol.Name);
            var structName = protocolName.ToPascal() + "Client";
            var channelName = protocolName.ToPascal() + "Channel";

            source.AddLine("use alloc::boxed::Box;");
            source.AddLine("use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, StormEvent};");
            source.AddLine("use uuid::Uuid;");
            source.AddLine($$"""use crate::channel::{{{channelName}}, ChannelMessageHeader, FromChannel};""");
            source.AddLine("use crate::from_client::*;");
            source.AddLine("use crate::from_server::*;");
            source.AddLine("use crate::message_ids::*;");
            source.AddLine("use alloc::vec::Vec;");
            source.AddBlank();

            var eventEnumBlock = source.AddBlock($"pub enum {structName}Event<'a>");
            foreach (var call in from.Values)
            {
                var callName = CasedString.FromSnake(call.Name);
                var (parametersType, _) = call.ToParametersType();
                var parameter = parametersType != null ? $"(&'a {parametersType.Name})" : "";
                eventEnumBlock.AddLine($"{callName.ToPascal()}{parameter},");
            }
            source.AddBlank();

            var observerTrait = source.AddBlock($"pub trait {structName}Observer");
            observerTrait.AddLine($"fn handle_{idl.Protocol.Name}_event(&mut self, channel_handle: ChannelHandle, event: {structName}Event);");
            source.AddBlank();

            var structBlock = source.AddBlock($"pub struct {structName}");
            structBlock.AddLine("channel_handle: ChannelHandle,");
            structBlock.AddLine($"channel: {channelName},");
            source.AddBlank();

            var implBlock = source.AddBlock($"impl {structName}");

            var connectBlock = implBlock.AddBlock("pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError>");
            connectBlock.AddLine($"let channel_handle = process.connect_to_service(\"{idl.Protocol.Name}\", None, None, None, {initialFromSize})?;");
            connectBlock.AddLine($"let channel = {channelName}::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), false);");
            var okBlock = connectBlock.AddBlock("Ok(Self");
            okBlock.Append = ")";
            okBlock.AddLine("channel_handle: channel_handle,");
            okBlock.AddLine("channel: channel,");
            implBlock.AddBlank();

            var processBlock = implBlock.AddBlock($"pub fn process_event(&self, process: &StormProcess, event: &StormEvent, observer: &mut impl {structName}Observer)");
            var matchBlock = processBlock.AddBlock("match event");
            var messageBlock = matchBlock.AddBlock("StormEvent::ChannelSignalled(channel_handle) =>");
            var ifBlock = messageBlock.AddBlock("if *channel_handle == self.channel_handle");
            //ifBlock.AddLine($"println!(\"{structName}: got event\");");
            var whileBlock = ifBlock.AddBlock("while let Some(message) = self.channel.find_message()");
            //whileBlock.AddLine("println!(\"found channel message\");");
            var unsafeBlock = whileBlock.AddBlock("unsafe");
            var innerMatchBlock = unsafeBlock.AddBlock("match (*message).message_id");
            foreach (var call in from.Values) {
                var callName = CasedString.FromSnake(call.Name);
                var (parametersType, parametersMessageName) = call.ToParametersType();
                var arm = innerMatchBlock.AddBlock($"{parametersMessageName} => ");
                //arm.AddLine($$"""println!("got {{parametersMessageName}} message");""");
                if (parametersType != null) {
                    arm.AddLine("let address = ChannelMessageHeader::get_payload_address(message);");
                    //arm.AddLine("""println!("found message at {:p}", address);""");
                    arm.AddLine($"{parametersType.Name}::reconstruct_at_inline(address);");
                    arm.AddLine($"let parameters = address as *const {parametersType.Name};");
                    arm.AddLine($"let request = {structName}Event::{callName.ToPascal()}(parameters.as_ref().unwrap());");
                    arm.AddLine($"observer.handle_{idl.Protocol.Name}_event(*channel_handle, request);");
                }
                else {
                    arm.AddLine($"observer.handle_{idl.Protocol.Name}_event(*channel_handle, {structName}Event::{callName.ToPascal()});");
                }
                arm.AddLine("self.channel.unlink_message(message, false);");
            }
            innerMatchBlock.AddLine("_ => {}");


            ifBlock.AddLine($"// observer.handle_{idl.Protocol.Name}_event(*channel_handle, event);");
            matchBlock.AddLine("_ => {}");
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

                var fnBlock = implBlock.AddBlock($"pub fn {call.Name}(&mut self{parameters}){returns}");
                fnBlock.AddLine($"let (call_id, message) = self.channel.prepare_message({parametersMessageName}, {(call.Type == IDLCall.CallType.SingleEvent ? "true" : "false")});");
                if (parametersType != null)
                {
                    fnBlock.AddLine("let payload = ChannelMessageHeader::get_payload_address(message);");
                    fnBlock.AddLine("let size = unsafe { parameters.write_at(payload) };");
                    fnBlock.AddLine("self.channel.commit_message(size);");
                    fnBlock.AddLine($"StormProcess::signal_channel(self.channel_handle);");
                }
                else
                {
                    fnBlock.AddLine("self.channel.commit_message(0);");
                }

                if (returnsType != null)
                {
                    fnBlock.AddBlank();
                    fnBlock.AddLine($"process.wait_for_channel_signal(self.channel_handle, 1000)?;");
                    fnBlock.AddBlank();

                    ifBlock = fnBlock.AddBlock($"if let Some(message) = self.channel.find_specific_message(call_id)");
                    ifBlock.AddLine("let payload = ChannelMessageHeader::get_payload_address(message);");
                    ifBlock.AddLine($$"""unsafe { {{returnsType.Name}}::reconstruct_at_inline(payload); }""");
                    ifBlock.AddLine($"let payload = payload as *mut {returnsType.Name};");
                    ifBlock.AddLine("Ok(FromChannel::new(&self.channel, message, unsafe { payload.as_ref().unwrap() }))");
                    var elseBlock = fnBlock.AddBlock("else");
                    elseBlock.AddLine("Err(StormError::NotFound)");
                }

                implBlock.AddBlank();
            }
        }
    }
}
