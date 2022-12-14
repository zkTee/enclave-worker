// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

extern crate dirs;
extern crate sgx_types;
extern crate sgx_urts;
extern crate log;
#[macro_use] extern crate lazy_static;

mod ecall_api;
mod ocall_api;
mod init;
mod seal;
mod enclave_main;
mod say;
mod ocall_impl;
mod db;

use std::thread;

fn main() {
    env_logger::init();

    let handler = thread::spawn(move || {
        enclave_main::main();
    });

    handler.join().unwrap();

    println!("🎅 ended!");
}

#[cfg(test)]
mod test {
    use super::*;
    use sgx_types::*;

    #[test]
    fn test() {
        let enclave = init::init_enclave().unwrap();

        let mut retval = 0usize;

        let result = unsafe { ecall_api::test_main_entrance(enclave.geteid(), &mut retval) };
    
        match result {
            sgx_status_t::SGX_SUCCESS => {}
            _ => {
                println!("[-] ECALL Enclave Failed {}!", result.as_str());
                return;
            }
        }
        assert_eq!(retval, 0);
    
        println!("[+] unit_test ended!");
    }
}