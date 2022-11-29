# enclave-worker

### Run
```shell
./x.sh
```

Intel SGX frontier.

- [x] seal-unseal
- [x] save to db & fetch
- [ ] enclave upgrade

---

### Use Log in Enclave
* To use env_logger, one must be sure the `TCSPolicy` is `0`.
* To use env_logger, one must include `sgx_env.edl` in the enclave's EDL file.
* In Cargo.toml, bring in log and env_logger:
```toml
log = { git = "https://github.com/mesalock-linux/log-sgx" }
env_logger = { git = "https://github.com/mesalock-linux/env_logger-sgx" }
```

* Import log and env_logger as usual:
```rust
#[macro_use] extern crate log
extern crate env_logger;
```
* Initialize and log as usual
```rust
env_logger::init();

info!("starting up");
```
* See the log output
```
$ make
$ cd bin
$ RUST_LOG=trace ./app
```
