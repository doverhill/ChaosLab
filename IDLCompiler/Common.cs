using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal class Common
    {
        public static string GetCallArguments(List<Field> fields)
        {
            return string.Join(", ", fields.Select(f => f.Name.ToSnake() + ": " + f.GetCallType()));
        }

        public static void ForEach<T>(List<T> collection, Action<T, bool> forEach)
        {
            for (var i = 0; i < collection.Count; i++)
            {
                if (i == collection.Count - 1)
                {
                    forEach(collection[i], true);
                }
                else
                {
                    forEach(collection[i], false);
                }
            }
        }
    }
}
