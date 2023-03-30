namespace Storm {
    public enum SyscallNumber {
        ServiceCreate = 100,
        ServiceSubscribe = 101,

        ChannelSignal = 200,

        EventWait = 300,

        ProcessCreate = 400,
        ProcessEmit = 401,
        ProcessReduceCapabilities = 402,

        TimerCreate = 500,

        Query = 600,

        // The following syscalls are unused when using hosted kernel
        ThreadCreate = 700,
        ThreadDestroy = 701,

        MemoryAllocate = 800,
        MemoryFree = 801,
        MemoryMap = 802,
        MemoryUnmap = 803,

        InterruptCreate = 900,
        InterruptDestroy = 901,

        HandleDestroy = 1000,
    }

    public enum SyscallProcessEmitType {
        Error = 1,
        Warning = 2,
        Information = 3,
        Debug = 4
    }
}
