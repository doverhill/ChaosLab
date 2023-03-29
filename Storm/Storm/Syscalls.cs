namespace Storm {
    public enum SyscallNumber {
        ServiceCreate = 100,
        ServiceDestroy = 101,
        ServiceSubscribe = 102,

        ChannelDestroy = 200,
        ChannelSignal = 201,

        EventWait = 300,

        ProcessCreate = 400,
        ProcessDestroy = 401,
        ProcessEmit = 402,
        // ProcessReduceCapabilities = 403 ??? to use once a process has done the things it needs its capabilties for, it can let them go for security

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
        InterruptDestroy = 901
    }

    public enum SyscallProcessEmitType {
        Error = 1,
        Warning = 2,
        Information = 3,
        Debug = 4
    }
}
