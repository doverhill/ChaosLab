using Core;
using VFS;

namespace ListFiles
{
    public class Application
    {
        public void Start()
        {
            var files = Directory.ListFiles(new DirectoryPath("/")).Require("Could not list files");

            var result = files.ForEach((item) => {
                Process.EmitDebug(item.Path.ToString());
                return false;
            });
            if (result.HasValue()) Process.EmitError(result.Value(), "Could not iterate files");

            Process.End();
        }
    }
}
