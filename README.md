# asbestos
asbestos is a tool for intercepting program I/O calls on Windows.

# How?

asbestos tries to accomplish its task by injecting a `.dll` file into the target process and hoooking calls to the Windows API.
The payload lives in `lib.rs`, while the main program lives in, well, `main.rs`.

# Linux? Mac?

I will implement support for platforms which I use regularly.
