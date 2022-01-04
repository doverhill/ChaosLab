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

            // append to crate
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
                TypeEmitter.Emit(output, idl, type, callType == CallType.Arguments, false);
            }
            else if (fieldsType == IDLDataSetType.List)
            {
                if (parsedFields.Count != 1) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": Only one field allowed when using List");
                var field = parsedFields[0];
                if (field.Type != Field.DataType.Type) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": Field must be custom type when using List");
                if (field.Name != null) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": Field can not be named when using List");

                IteratorTypeEmitter.Emit(idl, callName, field);
            }
            else if (fieldsType == IDLDataSetType.MixedList)
            {
                if (parsedFields.Count < 2) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": At least two fields needed when using MixedList");
                if (parsedFields.Any(f => f.Name != null)) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": Fields can not be named when using MixedList");
                if (parsedFields.Any(f => f.Type != Field.DataType.Type)) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": All fields must be custom type when using MixedList");

                IteratorTypeEmitter.EmitMixed(idl, callName, parsedFields);
            }
        }

        private static void EmitCall(StructuredWriter output, IDL idl, Direction direction, CasedString callName, List<string> parameters, List<string> returns, IDLDataSetType parametersType, IDLDataSetType returnsType)
        {
            var protocolName = CasedString.FromPascal(idl.Interface.Name);
            var messageName = protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + (direction == Direction.ClientToServer ? "_CLIENT_TO_SERVER" : "_SERVER_TO_CLIENT") + "_MESSAGE";

            var returnSignature = "";
            var parsedReturns = returns.Select(f => new Field(f, idl.Types)).ToList();
            if (returnsType == IDLDataSetType.ParameterSet)
            {
                if (returns.Count == 0)
                {
                    returnSignature = "()";
                }
                else if (returns.Count == 1)
                {
                    returnSignature = parsedReturns[0].GetStructType();
                }
                else
                {
                    returnSignature = callName.ToPascal() + CallType.Result.ToString();
                }
            }
            else if (returnsType == IDLDataSetType.List)
            {
                returnSignature = "crate::" + callName.ToPascal() + parsedReturns[0].TypeName.ToPascal() + "Iterator";
            }
            else if (returnsType == IDLDataSetType.MixedList)
            {
                returnSignature = "crate::" + callName.ToPascal() + "MixedIterator";
            }

            // write call functions
            var parsedParameters = parameters.Select(f => new Field(f, idl.Types)).ToList();
            if (parametersType == IDLDataSetType.ParameterSet)
            {
                output.WriteLine("pub fn call(channel_reference: Arc<Mutex<Channel>>, " + Common.GetCallArguments(parsedParameters) + ") -> Result<" + returnSignature + ", Error>", true);
                output.WriteLine("let mut channel = channel_reference.lock().unwrap();");
                output.WriteLine("channel.start();");
                if (parameters.Count > 0)
                {
                    output.WriteLine("let arguments = " + callName.ToPascal() + CallType.Arguments.ToString() + "::new(" + string.Join(", ", parsedParameters.Select(p => p.Name.ToSnake())) + ");");
                    output.WriteLine("channel.add_object(" + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + "_ARGUMENTS_OBJECT_ID, arguments);");
                }
            }
            else if (parametersType == IDLDataSetType.List || parametersType == IDLDataSetType.MixedList)
            {
                var parameterName = parametersType == IDLDataSetType.List ? 
                    parsedParameters[0].TypeName.ToPascal() :
                    (callName.ToPascal() + "Enum");
                var objectId = parametersType == IDLDataSetType.List ?
                    (protocolName.ToScreamingSnake() + "_" + parsedParameters[0].TypeName.ToScreamingSnake() + "_OBJECT_ID") :
                    (protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + "_ENUM_OBJECT_ID");

                output.WriteLine("pub fn call_vec(channel_reference: Arc<Mutex<Channel>>, objects: Vec<crate::" + parameterName + ">) -> Result<" + returnSignature + ", Error>", true);
                output.WriteLine("start(channel_reference.clone());");
                output.WriteLine("for object in objects", true);
                output.WriteLine("add(channel_reference.clone(), object);");
                output.CloseScope();
                output.WriteLine("call(channel_reference.clone())");
                output.CloseScope();
                output.BlankLine();

                output.WriteLine("pub fn start(channel_reference: Arc<Mutex<Channel>>)", true);
                output.WriteLine("let mut channel = channel_reference.lock().unwrap();");
                output.WriteLine("channel.start();");
                output.CloseScope();
                output.BlankLine();

                output.WriteLine("pub fn add(channel_reference: Arc<Mutex<Channel>>, object: crate::" + parameterName + ")", true);
                output.WriteLine("let mut channel = channel_reference.lock().unwrap();");
                output.WriteLine("channel.add_object(crate::" + objectId + ", object);");
                output.CloseScope();
                output.BlankLine();

                output.WriteLine("pub fn call(channel_reference: Arc<Mutex<Channel>>) -> Result<" + returnSignature + ", Error>", true);
                output.WriteLine("let channel = channel_reference.lock().unwrap();");
            }

            // write call and handle result
            if (returnsType == IDLDataSetType.ParameterSet)
            {
                if (returns.Count == 0)
                {
                    output.WriteLine("channel.call_sync(" + messageName + ", false, 1000)");
                }
                else
                {
                    output.WriteLine("match channel.call_sync(" + messageName + ", false, 1000)", true);
                    output.WriteLine("Ok(()) =>", true);
                    output.WriteLine("match channel.get_object::<" + callName.ToPascal() + CallType.Result.ToString() + ">(0, " + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + "_RESULT_OBJECT_ID)", true);
                    output.WriteLine("Ok(result) =>", true);
                    if (returns.Count == 1)
                    {
                        output.WriteLine("Ok(result.result)");
                    }
                    else
                    {
                        output.WriteLine("Ok(result)");
                    }
                    output.CloseScope(",");
                    output.WriteLine("Err(error) =>", true);
                    output.WriteLine("Err(error)");
                    output.CloseScope();
                    output.CloseScope();
                    output.CloseScope(",");
                    output.WriteLine("Err(error) =>", true);
                    output.WriteLine("Err(error)");
                    output.CloseScope();
                    output.CloseScope();
                }
            }
            else if (returnsType == IDLDataSetType.List || returnsType == IDLDataSetType.MixedList)
            {
                output.WriteLine("let result = channel.call_sync(" + messageName + ", false, 1000);");
                output.WriteLine("drop(channel);");
                output.WriteLine("match result", true);
                output.WriteLine("Ok(()) =>", true);
                if (returnsType == IDLDataSetType.List)
                {
                    output.WriteLine("Ok(crate::" + callName.ToPascal() + parsedReturns[0].TypeName.ToPascal() + "Iterator::new(channel_reference.clone()))");
                }
                else
                {
                    output.WriteLine("Ok(crate::" + callName.ToPascal() + "MixedIterator::new(channel_reference.clone()))");
                }
                output.CloseScope(",");
                output.WriteLine("Err(error) =>", true);
                output.WriteLine("Err(error)");
                output.CloseScope();
                output.CloseScope();
            }

            output.CloseScope();
        }

        public static void Emit(StructuredWriter output, Direction direction, IDL idl, IDLCall call)
        {
            var protocolName = CasedString.FromPascal(idl.Interface.Name);
            var callName = CasedString.FromPascal(call.Name);

            // write imports
            output.WriteLine("use library_chaos::{ Error, Channel, ChannelObject };");
            output.WriteLine("use core::{ mem, ptr, str, slice };");
            output.WriteLine("use std::sync::Arc;");
            output.WriteLine("use std::sync::Mutex;");
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

            // calls
            EmitCall(output, idl, direction, callName, parameters, returns, call.ParametersType, call.ReturnsType);

            // handle
            //EmitHandle(output)

        }
    }
}
