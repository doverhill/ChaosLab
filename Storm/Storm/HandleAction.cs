namespace Storm {
    public enum HandleAction
    {
        None = 0,
        ServiceConnected = 100,
        ServiceAvailable = 101,

        ChannelSignalled = 200,
        ChannelDestroyed = 201,

        TimerFired = 300,

        ProcessExited = 400,
    }
}
