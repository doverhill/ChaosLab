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
                "pub mod " + callName.ToSnake() + ";",
                "pub use " + callName.ToSnake() + "::" + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + (direction == Direction.ClientToServer ? "_CLIENT_TO_SERVER" : "_SERVER_TO_CLIENT") + "_MESSAGE;",
                ""
            });
        }

        public enum CallType
        {
            NotRelevant,
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
                if (field.Type != Field.DataType.Type) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": Field must be custom type when using List");
                if (field.Name != null) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": Field can not be named when using List");

                IteratorTypeEmitter.Emit(idl, callName, field);
            }
            else if (fieldsType == IDLDataSetType.MixedList)
            {
                if (parsedFields.Count < 2) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": At least two fields needed when using MixedList");
                if (parsedFields.Any(f => f.Name != null)) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": Fields can not be named when using MixedList");
                if (parsedFields.Any(f => f.Type != Field.DataType.Type)) throw new ArgumentException(callName.ToPascal() + callType.ToString() + ": All fields must be custom type when using MixedList");

                IteratorTypeEmitter.EmitMixed(idl, callName, parsedFields, callType);
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
                returnSignature = "crate::" + callName.ToPascal() + "MixedResultIterator";
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
                    (callName.ToPascal() + "ArgumentsEnum");
                var objectId = parametersType == IDLDataSetType.List ?
                    (protocolName.ToScreamingSnake() + "_" + parsedParameters[0].TypeName.ToScreamingSnake() + "_OBJECT_ID") :
                    (protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + "_ARGUMENTS_ENUM_OBJECT_ID");

                output.WriteLine("pub fn call(channel_reference: Arc<Mutex<Channel>>, objects: Vec<crate::" + parameterName + ">) -> Result<" + returnSignature + ", Error>", true);
                output.WriteLine("let mut channel = channel_reference.lock().unwrap();");
                output.WriteLine("channel.start();");
                output.WriteLine("for object in objects", true);
                output.WriteLine("channel.add_object(crate::" + objectId + ", object);");
                output.CloseScope();
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
                    output.WriteLine("Ok(crate::" + callName.ToPascal() + "MixedResultIterator::new(channel_reference.clone()))");
                }
                output.CloseScope(",");
                output.WriteLine("Err(error) =>", true);
                output.WriteLine("Err(error)");
                output.CloseScope();
                output.CloseScope();
            }

            output.CloseScope();
            output.BlankLine();
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
            output.WriteLine("pub fn handle(handler: &mut Box<dyn crate::" + idl.Interface.Name + (direction == Direction.ClientToServer ? "Server" : "Client") + "Implementation + Send>, channel_reference: Arc<Mutex<Channel>>)", true);

            if (call.ParametersType == IDLDataSetType.ParameterSet)
            {
                if (call.Parameters != null && call.Parameters.Count > 0)
                {
                    output.WriteLine("let mut channel = channel_reference.lock().unwrap();");
                    output.WriteLine("let arguments = match channel.get_object::<" + callName.ToPascal() + "Arguments>(0, " + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + "_ARGUMENTS_OBJECT_ID)", true);
                    output.WriteLine("Ok(arguments) =>", true);
                    output.WriteLine("arguments");
                    output.CloseScope(","); // Ok
                    output.WriteLine("Err(error) =>", true);
                    output.WriteLine("panic!(\"Failed to get arguments for " + callName.ToPascal() + ": {:?}\", error);");
                    output.CloseScope(); // Err
                    output.CloseScope(";"); // let match
                    output.BlankLine();

                    var argumentList = string.Join(", ", call.Parameters.Select(p => new Field(p, idl.Types)).Select(p => p.Type == Field.DataType.String ? ("&arguments." + p.Name.ToSnake()) : ("arguments." + p.Name.ToSnake())));

                    output.WriteLine("let result = handler." + callName.ToSnake() + "(" + argumentList + ");");
                }
                else
                {
                    output.WriteLine("let mut channel = channel_reference.lock().unwrap();");
                    output.WriteLine("let result = handler." + callName.ToSnake() + "();");
                }
            }
            else if (call.ParametersType == IDLDataSetType.List)
            {
                var parsedParameters = parameters.Select(p => new Field(p, idl.Types)).ToList();
                output.WriteLine("let iterator = crate::" + callName.ToPascal() + parsedParameters[0].TypeName.ToPascal() + "Iterator::new(channel_reference.clone());");
                output.WriteLine("let result = handler." + callName.ToSnake() + "(iterator);");
                output.WriteLine("let mut channel = channel_reference.lock().unwrap();");
            }
            else if (call.ParametersType == IDLDataSetType.MixedList)
            {
                output.WriteLine("let iterator = crate::" + callName.ToPascal() + "MixedArgumentsIterator::new(channel_reference.clone());");
                output.WriteLine("let result = handler." + callName.ToSnake() + "(iterator);");
                output.WriteLine("let mut channel = channel_reference.lock().unwrap();");
            }

            output.BlankLine();
            output.WriteLine("channel.start();");

            if (call.ReturnsType == IDLDataSetType.ParameterSet)
            {
                if (returns.Count == 0)
                {
                }
                else if (returns.Count == 1)
                {
                    output.WriteLine("let response = " + callName.ToPascal() + "Result::new(result);");
                    output.WriteLine("channel.add_object(" + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + "_RESULT_OBJECT_ID, response);");
                }
                else
                {
                    var argumentList = string.Join(", ", returns.Select(p => new Field(p, idl.Types)).Select(p => p.Type == Field.DataType.String ? ("&result." + p.Name.ToSnake()) : ("result." + p.Name.ToSnake())));
                    output.WriteLine("let response = " + callName.ToPascal() + "Result::new(" + argumentList + ");");
                    output.WriteLine("channel.add_object(" + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + "_RESULT_OBJECT_ID, response);");
                }
            }
            else if (call.ReturnsType == IDLDataSetType.List)
            {
                var parsedReturns = returns.Select(p => new Field(p, idl.Types)).ToList();
                output.WriteLine("for object in result", true);
                output.WriteLine("channel.add_object(crate::" + protocolName.ToScreamingSnake() + "_" + parsedReturns[0].TypeName.ToScreamingSnake() + "_OBJECT_ID, object);");
                output.CloseScope();
            }
            else if (call.ReturnsType == IDLDataSetType.MixedList)
            {
                output.WriteLine("for object in result", true);
                output.WriteLine("channel.add_object(crate::" + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + "_RESULT_ENUM_OBJECT_ID, object);");
                output.CloseScope();
            }

            output.WriteLine("channel.send(Channel::to_reply(" + protocolName.ToScreamingSnake() + "_" + callName.ToScreamingSnake() + "_" + (direction == Direction.ClientToServer ? "CLIENT_TO_SERVER" : "SERVER_TO_CLIENT") + "_MESSAGE, false));");

            output.CloseScope(); // fn handle
        }
    }
}
