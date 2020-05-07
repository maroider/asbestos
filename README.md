# asbestos
asbestos is a tool for interepting I/O from games. asbestos currently only works on Windows.

# How?

asbestos tries to accomplish its task by injecting a `.dll` file into the target process and hoooking calls to the Windows API.

# Current state?

Currently, asbestos is able to inject its payload into a target process and log what files are being accessed through a named socket.

# Linux? Mac?

I will implement support for platforms which I use regularly.

# Why nightly?

`detour` depends on some nightly features (`const_fn`, `unboxed_closures`, `abi_thiscall`) which makes its interface a lot nicer to use.
