using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.Linq;
using System.Security.Cryptography.X509Certificates;
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
            source.AddLine("use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, ServiceObserver, ChannelObserver};");
            source.AddLine("use uuid::Uuid;");
            source.AddLine($$"""use crate::channel::{{{channelName}}, ChannelMessageHeader};""");
            source.AddLine("use crate::from_client::*;");
            source.AddLine("use crate::from_server::*;");
            source.AddLine("use crate::MessageIds;");
            source.AddLine("use alloc::collections::BTreeMap;");
            source.AddLine("use alloc::vec::Vec;");
            source.AddBlank();

            var requestEnumBlock = source.AddBlock($"pub enum {structName}Request");
            foreach (var call in from.Values)
            {
                var callName = CasedString.FromSnake(call.Name);
                var (parametersType, _) = call.ToParametersType();
                var parameter = parametersType != null ? $"({parametersType.Name})" : "";
                requestEnumBlock.AddLine($"{callName.ToPascal()}{parameter},");
            }
            source.AddBlank();

            var observerTrait = source.AddBlock($"pub trait {structName}Observer");
            observerTrait.AddLine($"fn handle_{idl.Protocol.Name}_client_connected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);");
            observerTrait.AddLine($"fn handle_{idl.Protocol.Name}_client_disconnected(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle);");
            observerTrait.AddLine($"fn handle_{idl.Protocol.Name}_request(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, request: {structName}Request);");
            source.AddBlank();

            var structBlock = source.AddBlock($"pub struct {structName}<'a, T: {structName}Observer + PartialEq, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq>");
            structBlock.AddLine("service_handle: ServiceHandle,");
            structBlock.AddLine($"channels: BTreeMap<ChannelHandle, {channelName}>,");
            structBlock.AddLine("observers: Vec<&'a T>,");
            structBlock.AddLine("so: Option<&'a SO>,");
            structBlock.AddLine("co: Option<&'a CO>,");
            //structBlock.AddLine("on_client_connected: Option<Box<dyn Fn(ChannelHandle) + 'a>>,");
            //structBlock.AddLine("on_client_disconnected: Option<Box<dyn Fn(ChannelHandle) + 'a>>,");
            //foreach (var call in from.Values)
            //{
            //    structBlock.AddLine($"on_{call.Name}: Option<Box<dyn Fn(ChannelHandle) + 'a>>,");
            //}

            source.AddBlank();

            var implBlock = source.AddBlock($"impl<'a, T: {structName}Observer + PartialEq, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> {structName}<'a, T, SO, CO>");

            var createBlock = implBlock.AddBlock("pub fn create(process: &mut StormProcess<SO, CO>, vendor_name: &str, device_name: &str, device_id: Uuid) -> Result<Self, StormError>");
            createBlock.AddLine($"let service_handle = process.create_service(\"{idl.Protocol.Name}\", vendor_name, device_name, device_id)?;");
            //var connectHandler = createBlock.AddBlock("process.on_service_connected(service_handle, |_, channel_handle|");
            //connectHandler.Append = ");";
            //connectHandler.AddLine("StormProcess::emit_debug(\"X\");");
            var okBlock = createBlock.AddBlock("Ok(Self");
            okBlock.Append = ")";
            okBlock.AddLine("service_handle: service_handle,");
            okBlock.AddLine("channels: BTreeMap::new(),");
            okBlock.AddLine("observers: Vec::new(),");
            okBlock.AddLine("so: None,");
            okBlock.AddLine("co: None,");
            //okBlock.AddLine("on_client_connected: None,");
            //okBlock.AddLine("on_client_disconnected: None,");
            //foreach (var call in from.Values)
            //{
            //    okBlock.AddLine($"on_{call.Name}: None,");
            //}
            implBlock.AddBlank();

            //var onConnect = implBlock.AddBlock("pub fn on_client_connected(&mut self, handler: impl Fn(ChannelHandle) + 'a)");
            //onConnect.AddLine("self.on_client_connected = Some(Box::new(handler));");
            //implBlock.AddBlank();

            //onConnect = implBlock.AddBlock("pub fn clear_on_client_connected(&mut self)");
            //onConnect.AddLine("self.on_client_connected = None;");
            //implBlock.AddBlank();

            //onConnect = implBlock.AddBlock("pub fn on_client_disconnected(&mut self, handler: impl Fn(ChannelHandle) + 'a)");
            //onConnect.AddLine("self.on_client_disconnected = Some(Box::new(handler));");
            //implBlock.AddBlank();

            //onConnect = implBlock.AddBlock("pub fn clear_on_client_disconnected(&mut self)");
            //onConnect.AddLine("self.on_client_disconnected = None;");
            //implBlock.AddBlank();

            var attachBlock = implBlock.AddBlock("pub fn attach_observer(&mut self, observer: &'a T)");
            attachBlock.AddLine("self.observers.push(observer);");
            implBlock.AddBlank();

            var detachBlock = implBlock.AddBlock("pub fn detach_observer(&mut self, observer: &'a T)");
            var ifBlock = detachBlock.AddBlock("if let Some(index) = self.observers.iter().position(|x| *x == observer)");
            ifBlock.AddLine("self.observers.remove(index);");
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
                ifBlock = fnBlock.AddBlock("if let Some(channel) = self.channels.get(&channel_handle)");
                var unsafeBlock = ifBlock.AddBlock("unsafe");
                unsafeBlock.AddLine($"let message = channel.prepare_message(MessageIds::{parametersMessageName} as u64, {(call.Type == IDLCall.CallType.SingleEvent ? "true" : "false")});");
                if (parametersType != null)
                {
                    unsafeBlock.AddLine("let payload = ChannelMessageHeader::get_payload_address(message);");
                    unsafeBlock.AddLine("let size = parameters.write_at(payload);");
                    unsafeBlock.AddLine("channel.commit_message(size);");
                    unsafeBlock.AddLine($"StormProcess::<SO, CO>::send_channel_message(channel_handle, MessageIds::{parametersMessageName} as u64);");
                }
                else
                {
                    unsafeBlock.AddLine("self.channel.commit_message(0);");
                }

                implBlock.AddBlank();
            }

            //foreach (var call in from.Values)
            //{
            //    GenerateEventSetter(implBlock, call);
            //}

        }

        public static void GenerateClient(SourceGenerator source, IDL idl, Dictionary<string, IDLCall> from, Dictionary<string, IDLCall> to)
        {
            var protocolName = CasedString.FromSnake(idl.Protocol.Name);
            var structName = protocolName.ToPascal() + "Client";
            var channelName = protocolName.ToPascal() + "Channel";

            source.AddLine("use alloc::boxed::Box;");
            source.AddLine("use library_chaos::{StormProcess, ServiceHandle, ChannelHandle, StormError, ServiceObserver, ChannelObserver};");
            source.AddLine("use uuid::Uuid;");
            source.AddLine($$"""use crate::channel::{{{channelName}}, ChannelMessageHeader, FromChannel};""");
            source.AddLine("use crate::from_client::*;");
            source.AddLine("use crate::from_server::*;");
            source.AddLine("use crate::MessageIds;");
            source.AddLine("use alloc::vec::Vec;");
            source.AddBlank();

            var eventEnumBlock = source.AddBlock($"pub enum {structName}Event");
            foreach (var call in from.Values)
            {
                var callName = CasedString.FromSnake(call.Name);
                var (parametersType, _) = call.ToParametersType();
                var parameter = parametersType != null ? $"({parametersType.Name})" : "";
                eventEnumBlock.AddLine($"{callName.ToPascal()}{parameter},");
            }
            source.AddBlank();

            var observerTrait = source.AddBlock($"pub trait {structName}Observer");
            observerTrait.AddLine($"fn handle_{idl.Protocol.Name}_event(&mut self, service_handle: ServiceHandle, channel_handle: ChannelHandle, event: {structName}Event);");
            source.AddBlank();

            var structBlock = source.AddBlock($"pub struct {structName}<'a, T: {structName}Observer + PartialEq, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq>");
            structBlock.AddLine("channel_handle: ChannelHandle,");
            structBlock.AddLine($"channel: {channelName},");
            structBlock.AddLine("observers: Vec<&'a T>,");
            structBlock.AddLine("so: Option<&'a SO>,");
            structBlock.AddLine("co: Option<&'a CO>,");
            //structBlock.AddLine("channel_handle: ChannelHandle,");
            //structBlock.AddLine("channel_address: *mut u8,");
            //foreach (var call in from.Values)
            //{
            //    structBlock.AddLine($"on_{call.Name}: Option<Box<dyn Fn(ChannelHandle) + 'a>>,");
            //}

            source.AddBlank();

            var implBlock = source.AddBlock($"impl<'a, T: {structName}Observer + PartialEq, SO: ServiceObserver + PartialEq, CO: ChannelObserver + PartialEq> {structName}<'a, T, SO, CO>");

            var connectBlock = implBlock.AddBlock("pub fn connect_first(process: &mut StormProcess<SO, CO>) -> Result<Self, StormError>");
            connectBlock.AddLine($"let channel_handle = process.connect_to_service(\"{idl.Protocol.Name}\", None, None, None)?;");
            connectBlock.AddLine($$"""let channel = unsafe { {{channelName}}::new(process.get_channel_address(channel_handle).unwrap(), false) };""");
            var okBlock = connectBlock.AddBlock("Ok(Self");
            okBlock.Append = ")";
            //okBlock.AddLine("channel_handle: channel_handle,");
            //okBlock.AddLine("channel_address: process.get_channel_address(channel_handle).unwrap(),");
            okBlock.AddLine("channel_handle: channel_handle,");
            okBlock.AddLine("channel: channel,");
            okBlock.AddLine("observers: Vec::new(),");
            okBlock.AddLine("so: None,");
            okBlock.AddLine("co: None,");
            //foreach (var call in from.Values)
            //{
            //    okBlock.AddLine($"on_{call.Name}: None,");
            //}
            implBlock.AddBlank();

            var attachBlock = implBlock.AddBlock("pub fn attach_observer(&mut self, observer: &'a T)");
            attachBlock.AddLine("self.observers.push(observer);");
            implBlock.AddBlank();

            var detachBlock = implBlock.AddBlock("pub fn detach_observer(&mut self, observer: &'a T)");
            var ifBlock = detachBlock.AddBlock("if let Some(index) = self.observers.iter().position(|x| *x == observer)");
            ifBlock.AddLine("self.observers.remove(index);");
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
                    parameters = ", process: &StormProcess<SO, CO>";
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
                    unsafeBlock.AddLine($"StormProcess::<SO, CO>::send_channel_message(self.channel_handle, MessageIds::{parametersMessageName} as u64);");
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
                    ifBlock = unsafeBlock.AddBlock($"if let Some(message) = self.channel.find_specific_message(MessageIds::{returnsMessageName} as u64)");
                    ifBlock.AddLine("let payload = ChannelMessageHeader::get_payload_address(message);");
                    ifBlock.AddLine($"{returnsType.Name}::reconstruct_at_inline(payload);");
                    ifBlock.AddLine($"let payload = payload as *mut {returnsType.Name};");
                    ifBlock.AddLine($"Ok(FromChannel::new(payload.as_ref().unwrap()))");
                    var elseBlock = unsafeBlock.AddBlock("else");
                    elseBlock.AddLine("Err(StormError::NotFound)");
                }

                implBlock.AddBlank();
            }

            //foreach (var call in from.Values)
            //{
            //    GenerateEventSetter(implBlock, call);
            //}

        }

        //private static void GenerateEventSetter(SourceGenerator.SourceBlock implBlock, IDLCall call)
        //{
        //    var extraParameters = "";
        //    var fnBlock = implBlock.AddBlock($"pub fn on_{call.Name}(&mut self, handler: impl Fn(ChannelHandle{extraParameters}) + 'a)");
        //    fnBlock.AddLine($"self.on_{call.Name} = Some(Box::new(handler));");
        //    implBlock.AddBlank();

        //    fnBlock = implBlock.AddBlock($"pub fn clear_on_{call.Name}(&mut self)");
        //    fnBlock.AddLine($"self.on_{call.Name} = None;");
        //    implBlock.AddBlank();
        //}
    }
}
