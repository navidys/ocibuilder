# ocibuilder
OCI (Open Container Initiative) image builder written in Rust.

The project is under development and not ready for usage (feel free to contribute).

## Requirements

* fuse
* fuse-overlayfs

## Build binary

```shell
make
```

## Example

```shell
ctr1=$(ocibuilder from "${1:-quay.io/quay/busybox:latest}" | tail -1)
ocibuilder config --author navid --user apache $ctr1
ocibuilder config --port 4444/tcp $ctr1
```

## Commands

| Command    | Description |
| ---------- | ----------- |
| config     | Update image configuration settings.
| containers | List the working containers and their base images.
| from       | Creates a new working container either from scratch or using an image.
| images     | List images in local storage.
| mount      | Mounts a working container's root filesystem for manipulation.
| umount     | Unmounts the root file system of the specified working containers.
| pull       | Pull an image from the specified registry.
| reset      | Reset local storage.
