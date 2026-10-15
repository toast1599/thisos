# 0015. APIs and ABIs are designed for this system: inspired by others, never bound to them

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Every interface this OS defines can either follow an existing model for compatibility or be designed for this system. That covers system calls, IPC protocols, the program format, and the native API that std is built on.

Following an existing model buys existing software and familiarity, but it also brings that model's historical baggage. Unix's `fork` and `execve` are an example:
- `fork` copies a whole process only to replace it right away;
- it combines badly with threads;
- by default the child inherits everything the parent holds.

"A fork() in the road" (HotOS 2019) argues this case in detail. Fuchsia shows the alternative: it has no `fork`, and a program creates a new process and hands it exactly the handles it needs.

[0004](0004-not-unix.md) already decided that the OS isn't Unix-like, and that a Unix concept, when adopted, gets heavily adapted. With the microkernel ([0014](0014-microkernel.md)), every interface is being designed now.

## Options

1. Follow one existing model, such as POSIX, closely for compatibility.
2. Design each interface for this system, taking ideas from any existing system without being bound to any of them.

## Decision

Option 2.

- The OS's APIs and ABIs are designed for this system.
- They can be inspired by any existing system, whether Unix, Fuchsia, seL4, Windows or another.
- They are never locked into one particular model just for compatibility.
- A mechanism that exists elsewhere, such as `fork` and `execve`, is adopted only if it's the right design here. Other systems having it is not a reason.

## Consequences

- **No `fork` and `execve`.** Process creation won't copy them. What replaces them is open ([04-execution](../design/04-execution.md)).
- **Compatibility lives in separate layers.** Where compatibility with existing software is wanted ([0006](0006-non-rust-software.md)), it comes from separate user-mode layers built on the native API. Those layers adapt to the native API, never the other way round.
- **std translates; the native API doesn't bend.** Rust's std is the interface programs use ([0003](0003-rust-std-no-libc.md)). Some of its API reflects Unix: environment variables, file paths and process arguments. std's platform layer translates those onto the native API, and the native API isn't shaped to match them ([06-system](../design/06-system.md)).
- **Porting costs more.** Every Unix assumption a program makes has to be handled by a port or a compatibility layer. [0004](0004-not-unix.md) already accepted that cost for crates.
- **Every interface has to be designed here.** With no standard to follow, each interface is designed and documented from scratch. The single IDL and add-only versioning in [0007](0007-abi-calls-and-interfaces.md) (Proposed) are the tools for keeping that manageable.
- **It extends [0004](0004-not-unix.md) to every system.** What 0004 said about Unix now applies to every existing system.
