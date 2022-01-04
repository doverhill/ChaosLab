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
            output.WriteLine("static ref INSTANCES: Mutex<HashMap<Handle, Arc<Mutex<" + structName + ">>>> = ", true);
            output.WriteLine("Mutex::new(HashMap::new())");
            output.CloseScope(";");
            output.WriteLine("static ref CHANNELS: Mutex<HashMap<Handle, Handle>> = ", true);
            output.WriteLine("Mutex::new(HashMap::new())");
            output.CloseScope(";");
            output.WriteLine("static ref IMPLEMENTATIONS: Mutex<HashMap<Handle, Box<dyn " + structName + "Implementation + Send>>> = ", true);
            output.WriteLine("Mutex::new(HashMap::new())");
            output.CloseScope(";");
            output.CloseScope();
            output.BlankLine();

            // implementation trait
            output.WriteLine("pub trait " + structName + "Implementation", true);
            foreach (var call in inboundCalls)
            {
                var callName = CasedString.FromPascal(call.Name);
                var parsedParameters = call.Parameters.Select(p => new Field(p, idl.Types)).ToList();
                var parameters = Common.GetCallArguments(parsedParameters);
                output.WriteLine("fn " + callName.ToSnake() + "(&mut self" + (parameters.Length > 0 ? (", " + parameters) : "") + ")");
            }
            output.CloseScope();

        }
    }
}