<a name="0.3.0"></a>
## 0.3.0 (2022-10-28)

Features
* Add PoolID and description to Pools page
* Improve logging during validation
* Add support for managing pools
* Update README to be a little more helpful for Rust newbies
* Username validation against Slack and Custom Owners support
* Make compatible with Rust 2018 edition
* Added pools endpoint
* Change device pool filtering to query string
* Add drop down for filtering devices by pool
* Add device table searching and hover highlighting
* Make reservations randomly choose a device
* Add pools to devices in the UI
* Add a reservations api endpoint for reserving devices
* Add pools for devices to be in
* Add API for ending device reservations

Fixes
* Ensure updating a device leaves users on the same page
* Allow edit of pool when non-empty

<a name="0.2.3"></a>
## 0.2.3 (2018-08-02)


#### Features

*   Add better validation error messages ([28d28807](https://github.com/tismith/device-checkout-rs/commit/28d288072a14711143d99d972a6ae100c1ceb4ea))



<a name="0.2.2"></a>
## 0.2.2 (2018-07-27)


#### Features

*   Add Docker packaging support ([cfb84c8f](https://github.com/tismith/device-checkout-rs/commit/cfb84c8fdad8ca2a092a22536d8f1a360343ff3e))
*   Add more context on errors ([6b9be7a1](https://github.com/tismith/device-checkout-rs/commit/6b9be7a16f66dcbd720655edd358bd5f8d805712))



<a name="0.2.1"></a>
## 0.2.1 (2018-06-06)


#### Bug Fixes

*   Set the pool size to 1, avoid SQLITE_BUSY errors ([ebcd3287](https://github.com/tismith/device-checkout-rs/commit/ebcd3287ba13200c6015e39e22ab9e7c79ed7841))

#### Features

*   Add a snap package manifest for distribution ([58aa3b7e](https://github.com/tismith/device-checkout-rs/commit/58aa3b7e418c42d9e2e65ef751bbb19c9bc70103))
*   Add a --templates command-line argument ([abfb73a3](https://github.com/tismith/device-checkout-rs/commit/abfb73a32a7258f733542aef90e115ad2a38ff66))



<a name="0.2.0"></a>
## 0.2.0 (2018-05-22)


#### Features

*   use the full display width for tables ([e8c57c54](https://github.com/tismith/device-checkout-rs/commit/e8c57c54c7bb13ed540f782c955f9df77b16e7de))
*   add confirmation dialogs to deletes ([92264047](https://github.com/tismith/device-checkout-rs/commit/922640477e6367a3f5e7c3cae7c9fd339de96cea))
*   use cookies to pass status messages around ([2961372b](https://github.com/tismith/device-checkout-rs/commit/2961372b54b503d38060ceeb0e0eb4fea9f3556d))



<a name="0.1.0"></a>
## 0.1.0 Device Checkout (2018-05-15)




