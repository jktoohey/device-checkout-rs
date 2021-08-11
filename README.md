device-checkout-rs
==================
[![Build Status](https://travis-ci.org/tismith/device-checkout-rs.svg?branch=master)](https://travis-ci.org/tismith/device-checkout-rs)
[![codecov](https://codecov.io/gh/tismith/device-checkout-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/tismith/device-checkout-rs)
[![Snap Status](https://build.snapcraft.io/badge/tismith/device-checkout-rs.svg)](https://build.snapcraft.io/user/tismith/device-checkout-rs)
[![Dockerhub Status](https://img.shields.io/docker/build/tismith/device-checkout-rs.svg)](https://hub.docker.com/r/tismith/device-checkout-rs/)

Reimplementation of https://github.com/tismith/deviceCheckout in rust. Basically complete now. The HTTP API endpoints could use some more breadth, but the form based web ui is functional.

Since this is using `rocket` for the web framework, we need to use rust nightly, so we've pinned a working compiler using the rustc-toolchain file. Cargo build will pull down and install the correct compiler.

We use `diesel-migrations` to automatically build and migrate the database. No need to seed the database manually.

We are using:
-------------
* `rocket` for the web framework
* `diesel` as the database abstraction and orm
* `serde` for json serialization/deserialization
* `log` and `stderrlog` for configurable logging macros
* `clap` for commandline argument processing
* `failure` for error handling
* `assert_cli` for integration testing

Installation:
=============

Using `cargo`:
--------------

Install rustup and cargo: https://www.rust-lang.org/tools/install

```sh
cargo build
cargo run
```

Access help via command line args: `cargo run -- -h`

Using `snap`:
-------------

We're using [snapcraft](https://build.snapcraft.io) to automatically build snaps of device-checkout.

```sh
sudo snap install device-checkout
```

Using `docker`:
---------------

```sh
#Runs device-checkout on port 1234 with the database at /var/lib/devices.db
docker run -p 1234:8000 -v /var/lib:/var/lib/device-checkout tismith/device-checkout-rs
```

Development:
============
These need to be run whenever the toolchain is modified or the latest nightly is needed.
* Update rust toolchain: `rustup update`
* Update dependencies: `cargo update`

Build dependencies, run the app and tests.
* Build dependencies: `cargo build`
* Run application `cargo run`
* Run all tests: `cargo test`

Database Migration:
-------------------
If you are adding database tables, a migration needs to be added. These are created and generated with the Rust ORM *Diesel*.

* Install sqlite-dev on your system `sudo dnf install sqlite-devel`
* Install diesel CLI: `cargo install diesel_cli --no-default-features --features sqlite`
* Setup diesel with database in repo (stored in .env): `diesel setup`
* Create migration files for new tables: `diesel migration generate <new_feature>`
* Modify the migration files and schema
* Test the migration applies with `diesel migration run` and that it can be rolled back with `diesel migration redo`

Troubleshooting:
----------------

* Run with backtrace enabled: `RUST_BACKTRACE=1 cargo <command>`
* Increase logging: `cargo run -- -v` (Repeat for each log level)
  * Log levels: error, warning, info, debug, trace
