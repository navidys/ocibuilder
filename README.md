# ocibuilder
OCI (Open Container Initiative) image builder written in Rust.

The project is under development and not ready for usage (feel free to contribute).

## Requires

## Dependencies

### Fedora, CentOS, RHEL and related distributions

```console
$ sudo dnf install        \
    fuse                  \
    fuse-overlayfs        \
    pkg-config            \
    systemd-devel         \
    elfutils-libelf-devel \
    libseccomp-devel      \
    clang-devel           \
    openssl-devel
```

## Limitations

* sha384 and sha512 digests are not yet supported
* Build from Containerfile/Dockerfile is not yet supported

## Build binary

```shell
make
```

## Example

## Commands

| Command    | Description |
| ---------- | ----------- |
| reset      | Reset local storage.


## Run tests

## License

Licensed under the [MIT](LICENSE) license.
