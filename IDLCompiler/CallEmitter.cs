namespace IDLCompiler
{
    internal class CallEmitter
    {
        private static int CallId = 1;

        public enum Direction
        {
            ClientToServer,
            ServerToClient
        }

        private static string GetDirectoryName(Direction direction)
        {
            if (direction == Direction.ClientToServer)
            {
                return "client_to_server_calls";
            }
            else
            {
                return "server_to_client_calls";
            }
        }

        public static void Reset()
        {
            var directoryName = GetDirectoryName(Direction.ClientToServer);
            if (Directory.Exists(directoryName)) Directory.Delete(directoryName, true);
            Directory.CreateDirectory(directoryName);

            directoryName = GetDirectoryName(Direction.ServerToClient);
            if (Directory.Exists(directoryName)) Directory.Delete(directoryName, true);
            Directory.CreateDirectory(directoryName);
        }

        public static void Emit(Direction direction, IDL idl, IDLCall call)
        {
            var protocolName = CasedString.FromPascal(idl.Interface.Name);
            var callName = CasedString.FromPascal(call.Name);
            var directoryName = GetDirectoryName(direction);
            var stream = new StreamWriter(File.Create(directoryName + "/" + callName.ToSnake() + ".rs"));
            var output = new StructuredWriter(stream);
            Emit(output, direction, idl, call);
            stream.Close();

            // append to
            File.AppendAllLines(directoryName + "/mod.rs", new string[]
            {
                "mod " + callName.ToSnake() + ";",
                "pub use " + callName.ToSnake() + "::" + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + (direction == Direction.ClientToServer ? "_CLIENT_TO_SERVER" : "_SERVER_TO_CLIENT") + "_MESSAGE;",
                ""
            });
        }

        private enum CallType
        {
            Arguments,
            Result
        }

        private static void EmitType(StructuredWriter output, IDL idl, List<string> fields, IDLDataSetType fieldsType, CasedString callName, CallType callType)
        {
            var parsedFields = fields.Select(f => new Field(f, idl.Types)).ToList();

            if (fieldsType == IDLDataSetType.ParameterSet)
            {
                if (callType == CallType.Result && parsedFields.Count == 1)
                {
                    if (parsedFields.Any(f => f.Name != null)) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": Single field in result can not be named when using ParameterSet");
                    fields[0] = fields[0] + " Result";
                }
                else
                {
                    if (parsedFields.Any(f => f.Name == null)) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": All fields must be named when using ParameterSet");
                }

                var type = new IDLType
                {
                    Name = callName.ToPascal() + callType.ToString(),
                    Fields = fields
                };
                TypeEmitter.Emit(output, idl, type, false);
            }
            else if (fieldsType == IDLDataSetType.List)
            {
                if (parsedFields.Count != 1) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": Only one field allowed when using List");
                var field = parsedFields[0];
                if (field.Name != null) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": Field can not be named when using List");

                //IteratorEmitter()
            }
            else if (fieldsType == IDLDataSetType.MixedList)
            {
                if (parsedFields.Count < 2) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": At least two fields needed when using MixedList");
                if (parsedFields.Any(f => f.Name != null)) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": Fields can not be named when using MixedList");

            }

        }

        public static void Emit(StructuredWriter output, Direction direction, IDL idl, IDLCall call)
        {
            var protocolName = CasedString.FromPascal(idl.Interface.Name);
            var callName = CasedString.FromPascal(call.Name);

            // write imports
            output.WriteLine("use library_chaos::{ Error, Channel, ChannelObject };");
            output.WriteLine("use core::{ mem, ptr, str, slice };");
            output.WriteLine("use std::{ iter::Iterator, Arc, Mutex };");
            output.BlankLine();

            // call message
            output.WriteLine("pub const " + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + (direction == Direction.ClientToServer ? "_CLIENT_TO_SERVER" : "_SERVER_TO_CLIENT") + "_MESSAGE: u64 = " + CallId++ + ";");
            output.BlankLine();

            // call arguments?
            var parameters = call.Parameters ?? new List<string>();
            if (parameters.Count > 0)
            {
                EmitType(output, idl, parameters, call.ParametersType, callName, CallType.Arguments);
            }

            // call return?
            var returns = call.Returns ?? new List<string>();
            if (returns.Count > 0)
            {
                EmitType(output, idl, returns, call.ReturnsType, callName, CallType.Result);
            }


        }
    }
}
