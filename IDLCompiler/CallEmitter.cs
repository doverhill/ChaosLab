namespace IDLCompiler
{
    internal class CallEmitter
    {
        public static void Emit(StreamWriter writer, TypeEmitter typeEmitter, IDL idl, IDLCall call)
        {
            var emitter = new CommonEmitter(idl, writer, typeEmitter);
            emitter.WriteCall(call);
        }
    }
}
