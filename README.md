# ChaosLab

## Done

* multiple console support in host console server
* text drawing in console

## Todo

* Supporting IDL in console for shell to work
* Shell application
* service subscribe syscall to tell storm that we want to connect to a service, subsequent event_wait will return action when service is available -> cleaner startup
* no channel_signal queue in kernel
* add support for multiple services in default server code structure (self.clients not aware of service_handle atm)
* vfs server
* host fs server
* stormFs server
* server hotswap using service_subscribe syscall
* simple tornado IDL to build simple tornado apps: shell, filebrowser
* rip redox kernel and modify it to run chaos apps if license allows it
