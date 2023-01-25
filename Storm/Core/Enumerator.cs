using System;
using System.ComponentModel;

namespace Core
{
    public class Enumerator<T>
    {
        public ErrorOr<Optional<T>> GetNext()
        {
            return new ErrorOr<Optional<T>>(new Optional<T>());
        }

        public Optional<Error> ForEach(Func<T, bool> handler)
        {
            var item = GetNext();
            while (!item.IsError() && item.Value().HasValue())
            {
                var cancelled = handler(item.Value().Value());
                if (cancelled) return new Optional<Error>(Error.Cancelled);
                item = GetNext();
            }

            if (item.IsError()) return new Optional<Error>(item.Error());
            return new Optional<Error>();
        }
    }
}
