using System.Diagnostics;
using System.Runtime.CompilerServices;

namespace Storm {
    public static class ASSERT {
        [Conditional("DEBUG")]
        public static void That(bool condition, [CallerFilePath] string file = "", [CallerMemberName] string memberName = "", [CallerLineNumber] int lineNumber = 0) {
            if (!condition) {
                throw new Exception($"ASSERTION FAILED in member {memberName} in file {file}:{lineNumber}!");
            }
        }

        public static void NotReached([CallerFilePath] string file = "", [CallerMemberName] string memberName = "", [CallerLineNumber] int lineNumber = 0) {
            throw new Exception($"SHOULD NOT REACH in member {memberName} in file {file}:{lineNumber}!");
        }
    }
}
