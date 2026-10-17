# 0183. Stored access information lives on directories, as grants to stable user IDs

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

After a reboot, the filesystem decides by its own policy which capabilities to hand a session, and stored data carries access information for that policy ([0030](0030-filesystem-decides-access.md)). The login service gives each session an identity capability, which proves who is acting but grants nothing by itself. Services ask the login service whom it names ([0141](0141-identity-capability.md)).

## Options

1. **Grants on directories.**
2. **Access lists on every file,** checked on every open against the identity the opener presents.
3. **An owner only,** with sharing done by handing capabilities over at run time.

## Decision

Option 1.

- Access information lives on directories. Each directory records an owner and a list of grants. Each grant names a user and the rights the filesystem hands that user's sessions, in a capability for the directory.
- Users are named by a stable user ID that the login service assigns once and never reuses. That's what it answers when asked whom an identity capability names ([0141](0141-identity-capability.md)).
- Files carry no access information, only a record of who created them.
- Within a session, access comes only from capabilities ([0020](0020-capabilities.md)).

## Consequences

- **Identity decides what a session is handed, never each open** ([0141](0141-identity-capability.md)).
- **A directory is the unit of sharing.** Sharing one file means putting it in a directory shared with that user.
- **An owner changes a directory's grants through the filesystem,** by presenting their identity. The filesystem may act on identity, since it decides access by its own policy ([0141](0141-identity-capability.md)).
- **User IDs outlive sessions,** so stored records and audit logs can name users after a reboot.
- **0030's and 0141's questions on stored access information are answered.**
