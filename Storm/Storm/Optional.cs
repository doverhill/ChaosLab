namespace Storm {
    internal class Optional<T> {
        public bool HasValue;
        public T Value;

        public static Optional<T> WithValue(T value) {
            return new Optional<T> {
                HasValue = true,
                Value = value
            };
        }

        public static Optional<T> None() {
            return new Optional<T> {
                HasValue = false,
                Value = default
            };
        }
    }
}
