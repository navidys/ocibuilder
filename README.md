# ocibuilder
OCI (Open Container Initiative) image builder written in Rust.

OCIBuilder is using [yuki](https://github.com/youki-dev/youki) runtime.

The project is under development and not ready for usage (feel free to contribute).

## Requires

- Rust (see [here](https://www.rust-lang.org/tools/install)), edition 2021
- linux kernel ≥ 5.3

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

```shell
ctr1=$(ocibuilder from "${1:-quay.io/quay/busybox:latest}" | tail -1)
ocibuilder config --author navid --user apache $ctr1
ocibuilder config --port 4444/tcp $ctr1
ocibuilder run $ctr1 date > date.txt
ocibuilder commit $ctr1
```

## Commands

| Command    | Description |
| ---------- | ----------- |
| commit     | Creates an image from a working container.
| config     | Update image configuration settings.
| containers | List the working containers and their base images.
| from       | Creates a new working container either from scratch or using an image.
| images     | List images in local storage.
| mount      | Mounts a working container's root filesystem for manipulation.
| umount     | Unmounts the root file system of the specified working containers.
| pull       | Pull an image from the specified registry.
| reset      | Reset local storage.
| rm         | Remove one or more working containers.
| rmi        | Remove one or more images from local storage.
| run        | Run a command inside of the container.
| save       | Save an image to oci-archive tarball.

## License

Licensed under the [MIT](LICENSE) license.
