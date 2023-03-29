using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Storm {
    internal class ErrorOr<T> {
        public Error Error;
        public T Value;

        public bool IsError => Error != Error.None;

        public ErrorOr(T value) {
            Error = Error.None;
            Value = value;
        }

        public ErrorOr(Error error) {
            Error = error;
            Value = default;
        }
    }
}
