<p align="center">
  <a href="https://concilium-labs.org/">
    <img alt="Concilium" src="https://concilium-labs.org/dist/logo.png" width="200" />
  </a>
</p>

# Building

## **1. Install rustc, cargo and rustfmt.**

```bash
$ curl https://sh.rustup.rs -sSf | sh
$ source $HOME/.cargo/env
$ rustup component add rustfmt
```

When building the master branch, please make sure you are using the latest stable rust version by running:

```bash
$ rustup update
```

When building a specific release branch, you should check the rust version in `ci/rust-version.sh` and if necessary, install that version by running:
```bash
$ rustup install VERSION
```
Note that if this is not the latest rust version on your machine, cargo commands may require an [override](https://rust-lang.github.io/rustup/overrides.html) in order to use the correct version.

On Linux systems you may need to install libssl-dev, pkg-config, zlib1g-dev, protobuf etc.

On Ubuntu:
```bash
$ sudo apt-get update
$ sudo apt-get install libssl-dev libudev-dev pkg-config zlib1g-dev llvm clang cmake make libprotobuf-dev protobuf-compiler libclang-dev
```

On Fedora:
```bash
$ sudo dnf install openssl-devel systemd-devel pkg-config zlib-devel llvm clang cmake make protobuf-devel protobuf-compiler perl-core libclang-dev
```

## **2. Download the source code.**

```bash
$ git clone https://github.com/concilium-labs/concilium.git
$ cd concilium
```

## **3. Build.**

```bash
$ ./cargo build
```

> [!NOTE]
> Note that this builds a debug version that is **not suitable for running a testnet or mainnet validator**.
