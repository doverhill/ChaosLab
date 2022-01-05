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

        private static void EmitCall(StructuredWriter output, IDL idl, Side side, IDLCall call)
        {
            output.BlankLine();

            var callName = CasedString.FromPascal(call.Name);
            var parsedReturns = call.Returns?.Select(f => new Field(f, idl.Types)).ToList();
            var returnSignature = "";

            if (call.ReturnsType == IDLDataSetType.ParameterSet)
            {
                if (parsedReturns == null || parsedReturns.Count == 0)
                {
                    returnSignature = " -> Result<(), Error>";
                }
                else if (parsedReturns.Count == 1)
                {
                    returnSignature = " -> Result<" + parsedReturns[0].GetStructType() + ", Error>";
                }
                else
                {
                    returnSignature = " -> Result<(" + (parsedReturns == null ? "" : string.Join(", ", parsedReturns.Select(p => p.GetStructType()))) + "), Error>";
                }
            }
            else if (call.ReturnsType == IDLDataSetType.List)
            {
                returnSignature = " -> Result<crate::" + callName.ToPascal() + parsedReturns[0].TypeName.ToPascal() + "Iterator, Error>";
            }
            else if (call.ReturnsType == IDLDataSetType.MixedList)
            {
                returnSignature = " -> Result<crate::" + callName.ToPascal() + "MixedResultIterator, Error>";
            }

            var parsedParameters = call.Parameters?.Select(p => new Field(p, idl.Types)).ToList();

            if (call.ParametersType == IDLDataSetType.ParameterSet)
            {
                if (call.Parameters == null || call.Parameters.Count == 0)
                {
                    output.WriteLine("pub fn " + callName.ToSnake() + "(&self)" + returnSignature, true);
                    output.WriteLine("crate::" + (side == Side.Client ? "client_to_server_calls" : "server_to_client_calls") + "::" + callName.ToSnake() + "::call(self.channel_reference.clone())");
                    output.CloseScope(); // fn
                }
                else
                {
                    var parameterSignature = Common.GetCallArguments(parsedParameters);
                    output.WriteLine("pub fn " + callName.ToSnake() + "(&self, " + parameterSignature + ")" + returnSignature, true);
                    var callSignature = string.Join(", ", parsedParameters.Select(p => p.Name.ToSnake()));
                    output.WriteLine("crate::" + (side == Side.Client ? "client_to_server_calls" : "server_to_client_calls") + "::" + callName.ToSnake() + "::call(self.channel_reference.clone(), " + callSignature + ")");
                    output.CloseScope(); // fn
                }
            }
            else if (call.ParametersType == IDLDataSetType.List)
            {
                output.WriteLine("pub fn " + callName.ToSnake() + "(&self, objects: Vec<crate::" + parsedParameters[0].TypeName.ToPascal() + ">)" + returnSignature, true);
                output.WriteLine("crate::" + (side == Side.Client ? "client_to_server_calls" : "server_to_client_calls") + "::" + callName.ToSnake() + "::call(self.channel_reference.clone(), objects)");
                output.CloseScope(); // fn
            }
            else if (call.ParametersType == IDLDataSetType.MixedList)
            {
                output.WriteLine("pub fn " + callName.ToSnake() + "(&self, objects: Vec<crate::" + callName.ToPascal() + "ArgumentsEnum>)" + returnSignature, true);
                output.WriteLine("crate::" + (side == Side.Client ? "client_to_server_calls" : "server_to_client_calls") + "::" + callName.ToSnake() + "::call(self.channel_reference.clone(), objects)");
                output.CloseScope(); // fn
            }
        }

        public static void Emit(StructuredWriter output, Side side, IDL idl, List<IDLCall> inboundCalls, List<IDLCall> outboundCalls)
        {
            var protocolName = CasedString.FromPascal(idl.Interface.Name);
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
            if (side == Side.Client)
            {
                output.WriteLine("channel_reference: Arc<Mutex<Channel>>" + (inboundCalls != null && inboundCalls.Count > 0 ? "," : ""));
            }
            if (inboundCalls != null && inboundCalls.Count > 0)
            {
                output.WriteLine("pub implementation_factory: fn() -> Box<dyn " + structName + "Implementation + Send>");
            }
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
                output.WriteLine("let mut service = service_reference.lock().unwrap();");
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
                output.WriteLine("pub fn default(vendor: &str, description: &str, implementation_factory: fn() -> Box<dyn " + structName + "Implementation + Send>) -> Result<Arc<Mutex<" + structName + ">>, Error>", true);
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
                if (inboundCalls != null && inboundCalls.Count > 0)
                {
                    // from_channel
                    output.WriteLine("pub fn from_channel(channel_reference: Arc<Mutex<Channel>>) -> Self");

                }
                else
                {
                    // from_channel
                    output.WriteLine("pub fn from_channel(channel_reference: Arc<Mutex<Channel>>) -> Self", true);
                    output.WriteLine(structName, true);
                    output.WriteLine("channel_reference: channel_reference");
                    output.CloseScope();
                    output.CloseScope(); // fn from_channel
                    output.BlankLine();

                    // default
                    output.WriteLine("pub fn default() -> Result<Self, Error>", true);
                    output.WriteLine("match Service::connect(\"" + protocolName.ToPascal() + "\", None, None, None, 4096)", true);
                    output.WriteLine("Ok(channel_reference) =>", true);
                    output.WriteLine("let mut channel = channel_reference.lock().unwrap();");
                    output.WriteLine("channel.initialize(\"" + protocolName.ToPascal() + "\", " + idl.Interface.Version + ");");
                    output.WriteLine("drop(channel);");
                    output.BlankLine();
                    output.WriteLine("Ok(" + structName, true);
                    output.WriteLine("channel_reference: channel_reference");
                    output.CloseScope(")"); // Return
                    output.CloseScope(","); // Ok
                    output.WriteLine("Err(error) =>", true);
                    output.WriteLine("Process::emit_error(&error, \"Failed to connect to " + protocolName.ToPascal() + " service\").unwrap();");
                    output.WriteLine("Err(error)");
                    output.CloseScope(); // Err
                    output.CloseScope(); // match Service::connect
                    output.CloseScope(); // fn default
                    output.BlankLine();
                }
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
                    EmitCall(output, idl, side, call);
                }
            }

            output.CloseScope(); // impl
        }
    }
}