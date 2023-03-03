using System.Collections.Generic;
using System.ComponentModel.Design;
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
            source.AddLine("use crate::channel::*;");
            source.AddLine("use crate::message_ids::*;");
            source.AddLine("use alloc::collections::BTreeMap;");
            source.AddLine("use alloc::vec::Vec;");
            source.AddBlank();

            var requestEnumBlock = source.AddBlock($"pub enum {structName}Request<'a>");
            foreach (var call in from.Values)
            {
                var callName = CasedString.FromSnake(call.Name);
                var (parametersType, _) = call.ToParametersType();
                var parameter = parametersType != null ? $"(FromChannel<'a, &'a {parametersType.Name}>)" : "";
                requestEnumBlock.AddLine($"{callName.ToPascal()}{parameter},");
            }
            source.AddBlank();

            var eventEnumBlock = source.AddBlock($"pub enum {structName}ChannelEvent<'a>");
            eventEnumBlock.AddLine("ClientConnected(ServiceHandle, ChannelHandle),");
            eventEnumBlock.AddLine("ClientDisconnected(ServiceHandle, ChannelHandle),");
            eventEnumBlock.AddLine($"ClientRequest({structName}Request<'a>),");
            source.AddBlank();

            //var observerTrait = source.AddBlock($"pub trait {structName}Observer");
            //observerTrait.AddLine($"fn handle_{idl.Protocol.Name}_client_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);");
            //observerTrait.AddLine($"fn handle_{idl.Protocol.Name}_client_disconnected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);");
            //observerTrait.AddLine($"fn handle_{idl.Protocol.Name}_request(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, call_id: u64, request: {structName}Request);");
            //source.AddBlank();

            var structBlock = source.AddBlock($"pub struct {structName}");
            structBlock.AddLine("current_event: Option<StormEvent>,");
            structBlock.AddLine("service_handle: ServiceHandle,");
            structBlock.AddLine($"channels: BTreeMap<ChannelHandle, {channelName}>,");

            source.AddBlank();

            var implBlock = source.AddBlock($"impl {structName}");

            var createBlock = implBlock.AddBlock("pub fn create(process: &mut StormProcess, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError>");
            createBlock.AddLine($"let service_handle = process.create_service(\"{idl.Protocol.Name}\", vendor_name, device_name, device_id)?;");
            var okBlock = createBlock.AddBlock("Ok(Self");
            okBlock.Append = ")";
            okBlock.AddLine("current_event: None,");
            okBlock.AddLine("service_handle: service_handle,");
            okBlock.AddLine("channels: BTreeMap::new(),");
            implBlock.AddBlank();

            var registerBlock = implBlock.AddBlock("pub fn register_event(&mut self, event: StormEvent)");
            registerBlock.AddLine("self.current_event = Some(event);");
            implBlock.AddBlank();

            var processBlock = implBlock.AddBlock($"pub fn get_event(&mut self, process: &mut StormProcess) -> Option<{structName}ChannelEvent>");
            var ifBlock = processBlock.AddBlock("if let Some(current_event) = self.current_event");
            var matchBlock = ifBlock.AddBlock("match current_event");

            var connectBlock = matchBlock.AddBlock("StormEvent::ServiceConnected(service_handle, channel_handle) =>");
            //connectBlock.AddLine("println!(\"{:?} == {:?}?\", *service_handle, self.service_handle);");
            connectBlock.AddLine("self.current_event = None;");
            ifBlock = connectBlock.AddBlock("if service_handle == self.service_handle");
            ifBlock.AddLine($"println!(\"{structName}: client connected\");");
            ifBlock.AddLine($"process.initialize_channel(channel_handle, {initialFromSize});");
            ifBlock.AddLine($"let channel = {channelName}::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), true);");
            ifBlock.AddLine("self.channels.insert(channel_handle, channel);");
            ifBlock.AddLine($"Some({structName}ChannelEvent::ClientConnected(service_handle, channel_handle))");
            var elseBlock = connectBlock.AddBlock("else");
            elseBlock.AddLine("None");
            //ifBlock.AddLine($"observer.handle_{idl.Protocol.Name}_client_connected(*service_handle, *channel_handle);");

            var messageBlock = matchBlock.AddBlock("StormEvent::ChannelSignalled(channel_handle) =>");
            ifBlock = messageBlock.AddBlock("if let Some(channel) = self.channels.get(&channel_handle)");
            elseBlock = messageBlock.AddBlock("else");
            elseBlock.AddLine("self.current_event = None;");
            elseBlock.AddLine("None");
            //ifBlock.AddLine($"println!(\"{structName}: client request\");");
            var innerIfBlock = ifBlock.AddBlock("if let Some(message) = channel.find_message()");
            var innerElseBlock = ifBlock.AddBlock("else");
            innerElseBlock.AddLine("self.current_event = None;");
            innerElseBlock.AddLine("None");
            //whileBlock.AddLine("println!(\"found channel message\");");
            var unsafeBlock = innerIfBlock.AddBlock("unsafe");
            var innerMatchBlock = unsafeBlock.AddBlock("match (*message).message_id");
            foreach (var call in from.Values)
            {
                var callName = CasedString.FromSnake(call.Name);
                var (parametersType, parametersMessageName) = call.ToParametersType();
                var arm = innerMatchBlock.AddBlock($"{parametersMessageName} =>");
                arm.Append = ",";
                //arm.AddLine($$"""println!("got {{parametersMessageName}} message");""");
                if (parametersType != null) {
                    arm.AddLine("let address = ChannelMessageHeader::get_payload_address(message);");
                    arm.AddLine($"{parametersType.Name}::reconstruct_at_inline(address);");
                    arm.AddLine($"let parameters = address as *const {parametersType.Name};");
                    arm.AddLine($"let request = {structName}Request::{callName.ToPascal()}(FromChannel::new(channel, message, parameters.as_ref().unwrap()));");
                    arm.AddLine($"Some({structName}ChannelEvent::ClientRequest(request))");
                    //arm.AddLine($"observer.handle_{idl.Protocol.Name}_request(self.service_handle, *channel_handle, (*message).call_id, request);");
                }
                else {
                    arm.AddLine("channel.unlink_message(message, false);");
                    arm.AddLine($"Some({structName}ChannelEvent::ClientRequest({structName}Request::{callName.ToPascal()}))");
                    //arm.AddLine($"observer.handle_{idl.Protocol.Name}_request(self.service_handle, *channel_handle, (*message).call_id, {structName}Request::{callName.ToPascal()});");
                }
            }
            innerMatchBlock.AddLine($$"""_ => { panic!("{{structName}}: Unknown message received"); }""");

            var destroyBlock = matchBlock.AddBlock("StormEvent::ChannelDestroyed(channel_handle) =>");
            destroyBlock.AddLine("self.current_event = None;");
            ifBlock = destroyBlock.AddBlock("if let Some(_) = self.channels.get(&channel_handle)");
            ifBlock.AddLine($"println!(\"{structName}: client disconnected\");");
            ifBlock.AddLine($"Some({structName}ChannelEvent::ClientDisconnected(self.service_handle, channel_handle))");
            elseBlock = destroyBlock.AddBlock("else");
            elseBlock.AddLine("None");
            //ifBlock.AddLine($"observer.handle_{idl.Protocol.Name}_client_disconnected(self.service_handle, *channel_handle);");

            elseBlock = processBlock.AddBlock("else");
            elseBlock.AddLine("None");
            implBlock.AddBlank();

            foreach (var call in to.Values)
            {
                var (parametersType, parametersMessageName) = call.ToParametersType();
                //var (returnsType, returnsMessageName) = call.ToReturnsType(true);

                string parameters = "";
                if (parametersType != null)
                {
                    parameters = ", parameters: &" + parametersType.Name;
                }

                //string returns = "";
                //if (returnsType != null)
                //{
                //    returns = " -> " + returnsType.Name;
                //}

                var fnBlock = implBlock.AddBlock($"pub fn {call.Name}(&mut self, channel_handle: ChannelHandle{parameters})");
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

            foreach (var call in from.Values) {
                var (returnsType, returnsMessageName) = call.ToReturnsType(false);
                if (returnsType != null) {
                    var fnBlock = implBlock.AddBlock($"pub fn {call.Name}_reply(&mut self, channel_handle: ChannelHandle, call_id: u64, parameters: &{returnsType.Name})");
                    ifBlock = fnBlock.AddBlock("if let Some(channel) = self.channels.get_mut(&channel_handle)");
                    ifBlock.AddLine($"let (_, message) = channel.prepare_message({returnsMessageName}, false);");
                    ifBlock.AddLine("unsafe { (*message).call_id = call_id };");
                    ifBlock.AddLine("let payload = ChannelMessageHeader::get_payload_address(message);");
                    ifBlock.AddLine("let size = unsafe { parameters.write_at(payload) };");
                    ifBlock.AddLine("channel.commit_message(size);");
                    ifBlock.AddLine("StormProcess::signal_channel(channel_handle);");
                }
            }
        }

        public static void GenerateClient(SourceGenerator source, IDL idl, Dictionary<string, IDLCall> from, Dictionary<string, IDLCall> to, int initialFromSize, int intialToSize)
        {
            var protocolName = CasedString.FromSnake(idl.Protocol.Name);
            var structName = protocolName.ToPascal() + "Client";
            var channelName = protocolName.ToPascal() + "Channel";

            source.AddLine("use alloc::boxed::Box;");
            source.AddLine("use alloc::rc::Rc;");
            source.AddLine("use core::cell::RefCell;");
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
                var parameter = parametersType != null ? $"(FromChannel<'a, &'a {parametersType.Name}>)" : "";
                eventEnumBlock.AddLine($"{callName.ToPascal()}{parameter},");
            }
            source.AddBlank();

            eventEnumBlock = source.AddBlock($"pub enum {structName}ChannelEvent<'a>");
            eventEnumBlock.AddLine("ServerDisconnected(ChannelHandle),");
            eventEnumBlock.AddLine($"ServerEvent({structName}Event<'a>),");
            source.AddBlank();

            //var observerTrait = source.AddBlock($"pub trait {structName}Observer");
            //observerTrait.AddLine($"fn handle_{idl.Protocol.Name}_event(&mut self, channel_handle: ChannelHandle, event: {structName}Event);");
            //source.AddBlank();

            var structBlock = source.AddBlock($"pub struct {structName}");
            structBlock.AddLine("current_event: Option<StormEvent>,");
            structBlock.AddLine("channel_handle: ChannelHandle,");
            structBlock.AddLine($"channel: {channelName},");
            source.AddBlank();

            var implBlock = source.AddBlock($"impl {structName}");

            var connectBlock = implBlock.AddBlock("pub fn connect_first(process: &mut StormProcess) -> Result<Self, StormError>");
            connectBlock.AddLine($"let channel_handle = process.connect_to_service(\"{idl.Protocol.Name}\", None, None, None, {initialFromSize})?;");
            connectBlock.AddLine($"let channel = {channelName}::new(process.get_channel_address(channel_handle, 0).unwrap(), process.get_channel_address(channel_handle, 1).unwrap(), false);");
            var okBlock = connectBlock.AddBlock("Ok(Self");
            okBlock.Append = ")";
            okBlock.AddLine("current_event: None,");
            okBlock.AddLine("channel_handle: channel_handle,");
            okBlock.AddLine("channel: channel,");
            implBlock.AddBlank();

            var registerBlock = implBlock.AddBlock("pub fn register_event(&mut self, event: StormEvent)");
            registerBlock.AddLine("self.current_event = Some(event);");
            implBlock.AddBlank();

            var processBlock = implBlock.AddBlock($"pub fn get_event(&mut self, process: &StormProcess) -> Option<{structName}ChannelEvent>");
            var ifBlock = processBlock.AddBlock("if let Some(current_event) = self.current_event");
            var matchBlock = ifBlock.AddBlock("match current_event");

            var destroyBlock = matchBlock.AddBlock("StormEvent::ChannelDestroyed(channel_handle) =>");
            destroyBlock.AddLine("self.current_event = None;");
            ifBlock = destroyBlock.AddBlock("if channel_handle == self.channel_handle");
            ifBlock.AddLine($"Some({structName}ChannelEvent::ServerDisconnected(channel_handle))");
            var elseBlock = destroyBlock.AddBlock("else");
            elseBlock.AddLine("None");

            var messageBlock = matchBlock.AddBlock("StormEvent::ChannelSignalled(channel_handle) =>");
            ifBlock = messageBlock.AddBlock("if channel_handle == self.channel_handle");
            elseBlock = messageBlock.AddBlock("else");
            elseBlock.AddLine("self.current_event = None;");
            elseBlock.AddLine("None");
            //ifBlock.AddLine($"println!(\"{structName}: got event\");");
            var innerIfBlock = ifBlock.AddBlock("if let Some(message) = self.channel.find_message()");
            var innerElseBlock = ifBlock.AddBlock("else");
            innerElseBlock.AddLine("self.current_event = None;");
            innerElseBlock.AddLine("None");
            //whileBlock.AddLine("println!(\"found channel message\");");
            var unsafeBlock = innerIfBlock.AddBlock("unsafe");
            var innerMatchBlock = unsafeBlock.AddBlock("match (*message).message_id");
            foreach (var call in from.Values) {
                var callName = CasedString.FromSnake(call.Name);
                var (parametersType, parametersMessageName) = call.ToParametersType();
                var arm = innerMatchBlock.AddBlock($"{parametersMessageName} =>");
                arm.Append = ",";
                //arm.AddLine($$"""println!("got {{parametersMessageName}} message");""");
                if (parametersType != null) {
                    arm.AddLine("let address = ChannelMessageHeader::get_payload_address(message);");
                    //arm.AddLine("""println!("found message at {:p}", address);""");
                    arm.AddLine($"{parametersType.Name}::reconstruct_at_inline(address);");
                    arm.AddLine($"let parameters = address as *const {parametersType.Name};");
                    arm.AddLine($"let request = {structName}Event::{callName.ToPascal()}(FromChannel::new(&self.channel, message, parameters.as_ref().unwrap()));");
                    arm.AddLine($"Some({structName}ChannelEvent::ServerEvent(request))");
                    //arm.AddLine($"observer.handle_{idl.Protocol.Name}_event(*channel_handle, request);");
                }
                else {
                    arm.AddLine("self.channel.unlink_message(message, false);");
                    arm.AddLine($"Some({structName}ChannelEvent::ServerEvent({structName}Event::{callName.ToPascal()}))");
                    //arm.AddLine($"observer.handle_{idl.Protocol.Name}_event(*channel_handle, {structName}Event::{callName.ToPascal()});");
                }
            }
            innerMatchBlock.AddLine($$"""_ => { panic!("{{structName}}: Unknown message received"); }""");

            matchBlock.AddLine($$"""_ => { panic!("{{structName}}: Unexpected storm event type"); }""");

            elseBlock = processBlock.AddBlock("else");
            elseBlock.AddLine("None");
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
                    elseBlock = fnBlock.AddBlock("else");
                    elseBlock.AddLine("Err(StormError::NotFound)");
                }

                implBlock.AddBlank();
            }
        }
    }
}
