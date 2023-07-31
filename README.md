# thistle-yocto-build

This project is designed to wrap around the Yocto build system, to simplify usage and as well as managing common security features over the  built image.

[More information on our docs](https://docs.thistle.tech/thistle_yocto_build/getstarted/qemu)

Thistle-yocto-build works by accepting a configuration file in YAML format and outputting a fully built image.
The configuration file is based on the format used by the [kas](https://kas.readthedocs.io/en/latest/) project with extensions to specify the security components to be used by the assembled image.

### Features

* clone and checkout BitBake layers automatically
* build a Yocto image
* safe defaults (disabled ssh on production builds, etc..)
* default configurations for qemuarm64,  Raspberry Pi 4, BeagleBoneBlack
* basic security check & CVE audit post build
* direct integration with Thistle Update Client
* quality-of-life features such as one-line configuration flag to enable curl, openssl, etc..

Some features used in the repository are based off the [meta-thistle layer](https://github.com/thistletech/meta-thistle) - such as the Infineon TrustM linux tool integration.

### Example Config

```yaml
header:
  version: 11

target: base
machine: qemuarm64-thistle
distro: thistle-base

thistle-features:
  meta-thistle: e05a0ab0e3abfc3c8fcb5371fdffa451765826af
  curl:
    bin: true
    lib: true
    tls: openssl

repos:
  openembedded-core:
    url: git://git.openembedded.org/openembedded-core
    refspec: 54ee67b1a805a07288925d56e9956aabc23b6ab2
    layers:
      - meta

  meta-openembedded:
    url: git://git.openembedded.org/meta-openembedded
    refspec: kirkstone
    layers:
      - meta-oe
      - meta-python
      - meta-networking
      - meta-perl

local_conf_header:
  standard: |
    PACKAGE_CLASSES = "package_rpm"
    CONF_VERSION = "2"
```


### Usage

Dowload a released binary from the [releases page](https://github.com/thistletech/thistle-yocto-build/releases) or see the [build](#build) instructions below.

```sh
$ ./thistle-yocto-build --help
$ ./thistle-yocto-build gen-config qemu
$ ./thistle-yocto-build build --debug conf.yml
# Default image build time is ~ 45mins on a Ryzen 5 3600. Requires ~50GB of free storage.
```

### Build

_note_: the project only builds on x86_64 linux due to C dependency on [crypt(3) functions](https://github.com/pldubouilh/crypt3-sys)

```sh
# requires rust, x86_64-unknown-linux-musl target and clang
$ make release
```

### Docker Usage

Build image
```sh
$ docker build -t tyb .
```

Run build on docker image
```sh
$ mkdir build
$ cp samples/qemuarm64.yml build/
$ docker run --rm -v $(pwd)/build:/home/builder tyb thistle-yocto-build qemuarm64.yml build debug
```
