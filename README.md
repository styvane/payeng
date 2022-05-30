Payeng
======

[<img alt="https://github.com/styvane/payeng/actions/workflows/ci.yaml" src="https://img.shields.io/github/workflow/status/styvane/payeng/CI/main">](https://github.com/styvane/payeng/actions/workflows/ci.yaml) [<img alt="https://img.shields.io/github/license/styvane/payeng" src="https://img.shields.io/github/license/styvane/payeng">](LICENSE.txt) ![GitHub last commit (branch)](https://img.shields.io/github/last-commit/styvane/payeng/main)


Requirements
------------
The only requirement for this is Rust.

Additional you can install [bunyan-rs](https://crates.io/crates/bunyan) to better visualize the logs.

Supported Rust Versions
--------------------------
Payeng is built against the latest stable release. The minimum supported version is 1.61.
There is no guarantee to build on Rust versions earlier than the minimum supported version.

Run
---
Run the following command in two different terminals.

```bash
$ cargo run -- <your transaction file>  > <out of file>
```

To see the output log, set *RUST_LOG* to a valid log filter before running the binary.

```
$ RUST_LOG=info cargo run -- <your transaction file>  > <out of file> 

$ RUST_LOG=info cargo run -- <your transaction file>  > <out of file> | bunyan # if you have bunyan-rs installed.
```