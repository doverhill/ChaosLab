namespace Storm {
    public static class ASSERT
    {
        public static void That(bool condition)
        {
            if (!condition) throw new Exception("Condition assertion failed");
        }

        public static void NotReached()
        {
            throw new Exception("Not reached assertion failed");
        }
    }
}
