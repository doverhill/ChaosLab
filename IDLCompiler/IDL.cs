namespace IDLCompiler
{
    public class IDLInterface
    {
        public string Name; // the protocol name
        public int Version;
    }

    public class IDLType
    {
        public string Inherits;
        public string Name;
        public List<string> Fields;
    }

    public enum IDLDataSetType
    {
        // The standard way of calling a function and returning a single result
        // No custom types allowed anywhere
        // Example 1 (returns a single value): FileExists(string path, bool someFlag) -> bool;
        // Example 2 (returns multiple values): GetBasicInfo(string resourceName) -> GetBasicInfoResult; where GetBasicInfoResult(exists: bool, size: usize)
        ParameterSet,
        
        // Used for functions that calls using or returns a list of something
        // Only custom types allowed
        // Example 1 (standard function parameters, but returns a list): DirectoryList(string path, bool recurse) -> DirectoryChild[]
        // Example 2 (calls function with a list): ProcessResource(ResourceDescription[] resources) -> bool
        List,

        // Used for functions that need to send a mixed list of types
        // Only custom types allowed
        // Example 1 (used for parameters): StartUpdateRenderTree(); UpdateRenderTree_AddGuiWindow(GuiWindow window); UpdateRenderTree_AddGuiButton(GuiButton button); CommitUpdateRenderTree() -> UpdateRenderTreeStatusResult(something, somethingelse);
        // Example 2 (used for return): GetSomethingThatReturnsMixedList() -> Iterator; Iterator.ForEach(onGuiWindow => HandleGuiWindow, onGuiButton => HandleGuiButton)
        MixedList
    }

    public class IDLCall
    {
        public string Name;
        public IDLDataSetType ParametersType = IDLDataSetType.ParameterSet;
        public List<string> Parameters;
        public IDLDataSetType ReturnsType = IDLDataSetType.ParameterSet;
        public List<string> Returns;

        // Number of items to ensure space for in channel memory
        public int BatchSize = 64;
    }

    public class IDL
    {
        public IDLInterface Interface;
        public List<IDLType> Types;
        public List<IDLCall> ClientToServerCalls;
        public List<IDLCall> ServerToClientCalls;
    }
}
