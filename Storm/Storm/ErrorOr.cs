namespace Storm {
    internal class ErrorOr<T> {
        public bool IsError;
        public ErrorCode ErrorCode;
        public T Value;

        public static ErrorOr<T> Ok(T value) {
            return new ErrorOr<T> {
                IsError = false,
                ErrorCode = ErrorCode.None,
                Value = value
            };
        }

        public static ErrorOr<T> Error(ErrorCode error) {
            return new ErrorOr<T> {
                IsError = true,
                ErrorCode = error,
                Value = default
            };
        }
    }
}
