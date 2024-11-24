# ocibuilder
OCI (Open Container Initiative) image builder written in Rust.

OCIBuilder is using:
* [yuki](https://github.com/youki-dev/youki) runtime library within ocibuilder at moment.
* [rust-oci-client](https://github.com/oras-project/rust-oci-client) implements the OCI distribution specifictiion.

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
cntr=$(ocibuilder from "${1:-quay.io/quay/busybox:latest}" | tail -1)
ocibuilder config --author navid --working-dir /tmp $cntr
ocibuilder config --label "owner=ocibuilder" $cntr
ocibuilder run $cntr touch ocibuilder.txt
ocibuilder commit $cntr quay.io/ocibuilder/ocibuilder-test:latest
ocibuilder push quay.io/ocibuilder/ocibuilder-test:latest quay.io/ocibuilder/ocibuilder-test:latest
ocibuilder save -o /tmp/new_image.tar quay.io/ocibuilder/ocibuilder-test:latest

# Load the save image via podman
podman image load -i /tmp/new_image.tar
```

## Commands

| Command    | Description |
| ---------- | ----------- |
| commit     | Creates an image from a working container.
| config     | Update image configuration settings.
| containers | List the working containers and their base images.
| from       | Creates a new working container either from scratch or using an image.
| images     | List images in local storage.
| inspect    | Inspects a build container's or built image's configuration.
| mount      | Mounts a working container's root filesystem for manipulation.
| umount     | Unmounts the root file system of the specified working containers.
| pull       | Pull an image from the specified registry.
| push       | Pushes an image to a specified registry location.
| reset      | Reset local storage.
| rm         | Remove one or more working containers.
| rmi        | Remove one or more images from local storage.
| run        | Run a command inside of the container.
| save       | Save an image to oci-archive tarball.


## Run tests

`NOTE`: its required to run `ocibuilder reset` command before running the tests.

```shell
make validate
make test
```

## License

Licensed under the [MIT](LICENSE) license.
