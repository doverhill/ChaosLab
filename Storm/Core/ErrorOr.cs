namespace Core
{
    public class ErrorOr<T>
    {
        private bool isError;
        private Error error;
        private T value;

        public ErrorOr(Error error)
        {
            isError = true;
            this.error = error;
        }

        public ErrorOr(T value)
        {
            isError = false;
            this.value = value;
        }

        public bool IsError() => isError;

        public Error Error()
        {
            ASSERT.That(isError);
            return error;
        }

        public T Value()
        {
            ASSERT.That(!isError);
            return value;
        }

        public T Require(string errorMessage)
        {
            if (isError)
            {
                Process.EmitError(error, errorMessage);
                Process.End();
            }
            return value;
        }
    }
}
