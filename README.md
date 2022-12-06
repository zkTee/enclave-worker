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


---

### enclave.signed.so make 流程

待生成的 trusted/untrusted 代码：（bridge/proxy）
```shell
Enclave_EDL_Files := enclave-runtime/Enclave_t.c enclave-runtime/Enclave_t.h service/Enclave_u.c service/Enclave_u.h
```

---

1. `edge8r` 编译，生成 `proxy` 和 `bridge` 的 C++ 代码
```shell
######## EDL objects ########
$(Enclave_EDL_Files): $(SGX_EDGER8R) enclave-runtime/Enclave.edl
	$(SGX_EDGER8R) --trusted enclave-runtime/Enclave.edl --search-path $(SGX_SDK)/include --search-path $(CUSTOM_EDL_PATH) --trusted-dir enclave-runtime
	$(SGX_EDGER8R) --untrusted enclave-runtime/Enclave.edl --search-path $(SGX_SDK)/include --search-path $(CUSTOM_EDL_PATH) --untrusted-dir service
	@echo "GEN  =>  $(Enclave_EDL_Files)"
```

2. 编译C++代码, 生成 `libEnclave_u.a` 库文件
Worker_Enclave_u_Object :=service/libEnclave_u.a

```shell
service/Enclave_u.o: $(Enclave_EDL_Files)
	@$(CC) $(Worker_C_Flags) -c service/Enclave_u.c -o $@
	@echo "CC   <=  $<"

$(Worker_Enclave_u_Object): service/Enclave_u.o
	$(AR) rcsD $@ $^
	cp $(Worker_Enclave_u_Object) ./lib
```

3. 编译 `integritee-service`, `libEnclave_u.a` 是 service 的一部分
```shell
$(Worker_Name): $(Worker_Enclave_u_Object) $(SRC_Files)
	@echo
	@echo "Building the integritee-service"
	@SGX_SDK=$(SGX_SDK) SGX_MODE=$(SGX_MODE) cargo build -p integritee-service $(Worker_Rust_Flags)
	@echo "Cargo  =>  $@"
	cp $(Worker_Rust_Path)/integritee-service ./bin
```


4. 编译C++代码, 生成 `enclave.so` 库文件
RustEnclave_Name := enclave-runtime/enclave.so

```shell
######## Enclave objects ########
enclave-runtime/Enclave_t.o: $(Enclave_EDL_Files)
	@$(CC) $(RustEnclave_Compile_Flags) -c enclave-runtime/Enclave_t.c -o $@
	@echo "CC   <=  $<"

$(RustEnclave_Name): enclave enclave-runtime/Enclave_t.o
	@echo Compiling $(RustEnclave_Name)
	@$(CXX) enclave-runtime/Enclave_t.o -o $@ $(RustEnclave_Link_Flags)
	@echo "LINK =>  $@"
```

5. 生成最终的 `enclave.signed.so`, `enclave.so` 作为一部分嵌入

Signed_RustEnclave_Name := bin/enclave.signed.so

```shell
$(Signed_RustEnclave_Name): $(RustEnclave_Name)
	@echo
	@echo "Signing the enclave: $(SGX_ENCLAVE_MODE)"
	$(SGX_ENCLAVE_SIGNER) sign -key $(SGX_SIGN_KEY) -enclave $(RustEnclave_Name) -out $@ -config $(SGX_ENCLAVE_CONFIG)
	@echo "SIGN =>  $@"
	@echo
	@echo "Enclave is in $(SGX_ENCLAVE_MODE)"
```

