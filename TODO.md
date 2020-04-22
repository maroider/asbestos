# Wow54

asbestos can currently only hook processes compiled for the same architecture as itself.
This will likely require a build script on either asbestos or asbestos-cli to build both 64-bit and 32-bit payloads.

`IsWow64Process2` is likely the function to use to detect if a given process is 64-bit or 32-bit.
`IsWow64GuestMachineSupported` may also be of use here.

# Potential Function Hooks

* `ShellExecute` and `ShellExecuteEx`. Would likely require hooking `ShellExecute` and redirecting it to `ShellExecuteEx`.
