namespace Core
{
    public class Optional<T>
    {
        private bool hasValue;
        private T value;

        public Optional()
        {
            hasValue = false;
        }

        public Optional(T value)
        {
            hasValue = true;
            this.value = value;
        }

        public bool HasValue() => hasValue;

        public T Value()
        {
            ASSERT.That(hasValue);
            return value;
        }
    }
}
