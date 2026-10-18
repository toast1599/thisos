# 0230. Interfaces that third-party programs reach are released with the SDK

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Each interface's version 1 is fixed when it's first released, and the roadmap says when that is ([0148](0148-freeze-covers-rules.md)). Releasing a version adds its frozen copy to the repository ([0196](0196-released-interface-copies.md)), and from then on the interface only grows ([0143](0143-add-only-versioning.md)). Everything in the system image updates together ([0042](0042-updates.md)). Third-party programs build with the published target specs, std's patch and the `thisos` crate ([0187](0187-pinned-stable-toolchain.md), [0195](0195-bindings-safe-output.md)).

## Options

1. **With the SDK.** The interfaces third-party programs reach are released when programs can first be built against them.
2. **At the end of each step** that builds an interface.
3. **All at the first release,** internal ones included.

## Decision

Option 1.

- At step 23, the interfaces that third-party programs can reach are released: the system library's ([0117](0117-system-library.md)), the startup data ([0146](0146-startup-data-block.md)), and the protocols behind std and the `thisos` crate ([0145](0145-std-over-native-api.md), [0178](0178-native-api-crate.md)).
- An interface used only inside the system image, such as the boot manifest ([0160](0160-root-task-starts-four.md)), the boot information ([0159](0159-root-task-boot-information.md)) or a driver's protocol, is released when a third-party program first needs it.
- Until its release, an interface changes with the image, within the frozen design ([0148](0148-freeze-covers-rules.md)).

## Consequences

- **Interfaces are built and used by the OS itself before they're fixed,** as 0148 intends.
- **Internal interfaces keep their freedom** while only the image depends on them ([0042](0042-updates.md)).
- **The compatibility checker guards each interface from its release on** ([0196](0196-released-interface-copies.md)). It's built at step 6, and tested against copies made for its tests.
- **0148's question on when interfaces are released is answered,** and so is 05's.
