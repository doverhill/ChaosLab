# ChaosLab

## Done

* multiple console support in host console server
* text drawing in console
* Supporting IDL in console for shell to work
* Shell application
* add support for multiple services in default server code structure (self.clients not aware of service_handle atm)
* host fs server

## Todo

* let console server implement both Console protocol and Data protocol
    * so that shell can connect using Data
    * tornado can connect using Console
    * terminal application can expose Data service to let shell run in window
* vfs server

* simple tornado IDL to build simple tornado apps:
    * shell
    * file browser
    * text editor

* service subscribe syscall to tell storm that we want to connect to a service, subsequent event_wait will return action when service is available -> cleaner startup
* no channel_signal queue in kernel
* stormFs server
* server hotswap using service_subscribe syscall
* rip redox kernel and modify it to run chaos apps if license allows it
* capabilities / claims / rights