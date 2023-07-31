from archlinux:base-devel

# pull deps
run pacman -Sy
run pacman --noconfirm -S base-devel git diffstat unzip texinfo python chrpath wget xterm rpcsvc-proto socat cpio inetutils rustup clang
run pacman --noconfirm -S musl

run rustup install stable
run rustup target add x86_64-unknown-linux-musl

# cache optimisation
run cargo search minisign

# set local
run echo "en_US.UTF-8 UTF-8" > /etc/locale.gen
run locale-gen

# install
run wget -q https://downloads.thistle.tech/thistle-build/edge/thistle-build

# make builder user
run useradd -ms /bin/bash builder
user builder
workdir /home/builder

# dummy creds for debug build
env THISTLE_YOCTO_BUILD_USERNAME a
env THISTLE_YOCTO_BUILD_PASSWORD a

# set entrypoint with help
cmd ["thistle-yocto-build", "--help"]

## Build
# $ docker build -t tyb .

## Run
# $ docker run tyb thistle-yocto-build --help

## Build Image
# $