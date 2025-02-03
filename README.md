# How To Work

## Unexpected rebuilds
Some issues we've seen historically which can cause crates to get rebuilt are:

* A build script prints `cargo:rerun-if-changed=foo` where `foo` is a file that
  doesn't exist and nothing generates it.

* Two successive Cargo builds may differ in the set of features enabled for some
  dependencies.

* Some filesystems exhibit unusual behavior around timestamps.

* A concurrent build process is either deleting artifacts or modifying files.

* .cargo/config in vendor workspaces.