using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal class ClientServerEmitter
    {
        public enum Side
        {
            Client,
            Server
        }

        public static void Emit(Side side, IDL idl, List<IDLCall> inboundCalls, List<IDLCall> outboundCalls)
        {
            var filename = side == Side.Client ? "client.rs" : "server.rs";
            var stream = new StreamWriter(File.Create(filename));
            var output = new StructuredWriter(stream);
            Emit(output, side, idl, inboundCalls, outboundCalls);
            stream.Close();
        }

        private static void EmitTraitCall(StructuredWriter output, IDL idl, IDLCall call)
        {
            var callName = CasedString.FromPascal(call.Name);
            var parsedReturns = call.Returns?.Select(f => new Field(f, idl.Types)).ToList();
            var returnSignature = "";

            if (call.ReturnsType == IDLDataSetType.ParameterSet)
            {
                if (parsedReturns == null || parsedReturns.Count == 0)
                {
                    returnSignature = "";
                }
                else if (parsedReturns.Count == 1)
                {
                    returnSignature = " -> " + parsedReturns[0].GetStructType();
                }
                else
                {
                    returnSignature = " -> (" + (parsedReturns == null ? "" : string.Join(", ", parsedReturns.Select(p => p.GetStructType()))) + ")";
                }
            }
            else if (call.ReturnsType == IDLDataSetType.List)
            {
                returnSignature = " -> Vec<crate::" + parsedReturns[0].TypeName.ToPascal() + ">";
            }
            else if (call.ReturnsType == IDLDataSetType.MixedList)
            {
                returnSignature = " -> Vec<crate::" + callName.ToPascal() + "ResultEnum>";
            }

            var parsedParameters = call.Parameters?.Select(p => new Field(p, idl.Types)).ToList();

            if (call.ParametersType == IDLDataSetType.ParameterSet)
            {
                if (call.Parameters == null || call.Parameters.Count == 0)
                {
                    output.WriteLine("fn " + callName.ToSnake() + "(&mut self)" + returnSignature + ";");
                }
                else
                {
                    var parameterSignature = Common.GetCallArguments(parsedParameters);
                    output.WriteLine("fn " + callName.ToSnake() + "(&mut self, " + parameterSignature + ")" + returnSignature + ";");
                }
            }
            else if (call.ParametersType == IDLDataSetType.List)
            {
                output.WriteLine("fn " + callName.ToSnake() + "(&mut self, objects: crate::" + callName.ToPascal() + parsedParameters[0].TypeName.ToPascal() + "Iterator)" + returnSignature + ";");
            }
            else if (call.ParametersType == IDLDataSetType.MixedList)
            {
                output.WriteLine("fn " + callName.ToSnake() + "(&mut self, objects: crate::" + callName.ToPascal() + "MixedArgumentsIterator)" + returnSignature + ";");
            }
        }

        public static void Emit(StructuredWriter output, Side side, IDL idl, List<IDLCall> inboundCalls, List<IDLCall> outboundCalls)
        {
            var structName = idl.Interface.Name + (side == Side.Client ? "Client" : "Server");

            output.WriteLine("extern crate library_chaos;");
            output.BlankLine();

            output.WriteLine("use std::sync::Arc;");
            output.WriteLine("use std::sync::Mutex;");
            output.WriteLine("use std::collections::HashMap;");
            output.WriteLine("use library_chaos::{ Channel, Error, Process, Service, Handle };");
            output.WriteLine("use uuid::Uuid;");
            output.BlankLine();

            // statics
            output.WriteLine("lazy_static!", true);
            output.WriteLine("static ref INSTANCES: Mutex<HashMap<Handle, Arc<Mutex<" + structName + ">>>> =", true);
            output.WriteLine("Mutex::new(HashMap::new())");
            output.CloseScope(";");
            output.WriteLine("static ref CHANNELS: Mutex<HashMap<Handle, Handle>> =", true);
            output.WriteLine("Mutex::new(HashMap::new())");
            output.CloseScope(";");
            output.WriteLine("static ref IMPLEMENTATIONS: Mutex<HashMap<Handle, Box<dyn " + structName + "Implementation + Send>>> =", true);
            output.WriteLine("Mutex::new(HashMap::new())");
            output.CloseScope(";");
            output.CloseScope();
            output.BlankLine();

            // implementation trait
            output.WriteLine("pub trait " + structName + "Implementation", true);
            if (inboundCalls != null && inboundCalls.Count > 0)
            {
                foreach (var call in inboundCalls)
                {
                    EmitTraitCall(output, idl, call);
                }
            }
            output.CloseScope();
            output.BlankLine();

            // struct
            output.WriteLine("pub struct " + structName, true);
            output.WriteLine("pub implementation_factory: fn() -> Box<dyn " + structName + "Implementation + Send>");
            output.CloseScope();
            output.BlankLine();

            // impl
            output.WriteLine("impl " + structName, true);

            if (side == Side.Server)
            {
                // from_service
                output.WriteLine("pub fn from_service(service_reference: Arc<Mutex<Service>>, implementation_factory: fn() -> Box<dyn " + structName + "Implementation + Send>) -> Arc<Mutex<" + structName + ">>", true);
                output.WriteLine("let instance = " + structName, true);
                output.WriteLine("implementation_factory: implementation_factory");
                output.CloseScope(";");
                output.BlankLine();
                output.WriteLine("let service = service_reference.lock().unwrap();");
                output.WriteLine("service.on_connect(Self::handle_connect).unwrap();");
                output.BlankLine();
                output.WriteLine("let instance_reference = Arc::new(Mutex::new(instance));");
                output.WriteLine("let mut instances = INSTANCES.lock().unwrap();");
                output.WriteLine("instances.insert(service.handle, instance_reference.clone());");
                output.BlankLine();
                output.WriteLine("instance_reference");
                output.CloseScope(); // fn from_service
                output.BlankLine();

                // default
                output.WriteLine("pub fn default(vendor: &str, description: &str, implementation_factory: fn() -> Box<dyn " + structName + "Implementation + Send>) -> Arc<Mutex<" + structName + ">>", true);
                output.WriteLine("match Service::create(\"" + idl.Interface.Name + "\", vendor, description, Uuid::parse_str(\"00000000-0000-0000-0000-000000000000\").unwrap())", true);
                output.WriteLine("Ok(service_reference) =>", true);
                output.WriteLine("Ok(Self::from_service(service_reference, implementation_factory))");
                output.CloseScope(","); // Ok
                output.WriteLine("Err(error) =>", true);
                output.WriteLine("Process::emit_error(&error, \"Failed to create service\").unwrap();");
                output.WriteLine("Err(error)");
                output.CloseScope(); // Err
                output.CloseScope(); // match Service::create
                output.CloseScope(); // fn default
                output.BlankLine();

                // handle_connect
                output.WriteLine("fn handle_connect(service_reference: &Arc<Mutex<Service>>, channel_reference: Arc<Mutex<Channel>>)", true);
                output.WriteLine("let service = service_reference.lock().unwrap();");
                output.WriteLine("let instances = INSTANCES.lock().unwrap();");
                output.WriteLine("if let Some(instance_reference) = instances.get(&service.handle)", true);
                output.WriteLine("let mut channels = CHANNELS.lock().unwrap();");
                output.WriteLine("let mut channel = channel_reference.lock().unwrap();");
                output.WriteLine("channels.insert(channel.handle, service.handle);");
                output.WriteLine("channel.on_message(Self::handle_message).unwrap();");
                output.WriteLine("let mut implementations = IMPLEMENTATIONS.lock().unwrap();");
                output.WriteLine("let instance = instance_reference.lock().unwrap();");
                output.WriteLine("let implementation = (instance.implementation_factory)();");
                output.WriteLine("implementations.insert(channel.handle, implementation);");
                output.CloseScope(); // if let
                output.CloseScope(); // fn handle_connect
                output.BlankLine();
            }
            else
            {
                // client side

            }

            // handle_message
            output.WriteLine("fn handle_message(channel_reference: Arc<Mutex<Channel>>, message: u64)", true);
            output.WriteLine("let channel = channel_reference.lock().unwrap();");
            output.WriteLine("let channel_handle = channel.handle;");
            output.WriteLine("drop(channel);");
            output.BlankLine();
            output.WriteLine("let mut implementations = IMPLEMENTATIONS.lock().unwrap();");
            output.WriteLine("if let Some(implementation) = implementations.get_mut(&channel_handle)", true);
            output.WriteLine("match message", true);

            var protocolName = CasedString.FromPascal(idl.Interface.Name);
            if (inboundCalls != null)
            {
                foreach (var call in inboundCalls)
                {
                    var callName = CasedString.FromPascal(call.Name);
                    output.WriteLine("crate::" + (side == Side.Client ? "server_to_client_calls" : "client_to_server_calls") + "::" + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + "_" + (side == Side.Client ? "SERVER_TO_CLIENT" : "CLIENT_TO_SERVER") + "_MESSAGE =>", true);
                    output.WriteLine("crate::" + (side == Side.Client ? "server_to_client_calls" : "client_to_server_calls") + "::" + callName.ToSnake() + "::handle(implementation, channel_reference);");
                    output.CloseScope();
                }
            }

            output.WriteLine("_ =>", true);
            output.WriteLine("panic!(\"Unknown message {} received for protocol " + idl.Interface.Name + "\", message);");
            output.CloseScope(); // _ =>

            output.CloseScope(); // match message
            output.CloseScope(); // if let
            output.CloseScope(); // fn handle_message

            if (outboundCalls != null)
            {
                foreach (var call in outboundCalls)
                {
                    var callName = CasedString.FromPascal(call.Name);
                    output.BlankLine();
                    output.WriteLine("pub fn " + callName.ToSnake() + "() {}");
                }
            }

            output.CloseScope(); // impl
        }
    }
}