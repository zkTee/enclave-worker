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

#![crate_name = "enclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
extern crate sgx_tseal;

#[cfg(not(target_env = "sgx"))]
#[macro_use] extern crate sgx_tstd as std;
extern crate sgx_rand;

#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
extern crate sgx_tunittest;

use sgx_types::*;
use std::string::String;
use std::vec::{Vec};
use std::io::{self, Write};
use std::slice;
use sgx_tseal::{SgxSealedData};
use sgx_rand::{Rng, StdRng};
use sgx_tunittest::*;
use sgx_types::marker::ContiguousMemory;

mod test_seal;
use test_seal::*;

mod save;
mod ocall_api;

pub type Bytes = Vec<u8>;
pub const MEGA_BYTE: usize = 1_000_000;
pub const SCRATCH_PAD_SIZE: usize = MEGA_BYTE * 1;
pub const U32_NUM_BYTES: usize = 4;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Item {
    name: String,
}
impl Item {
    pub fn new(name: String) -> Self {
        Item { name }
    }
}

#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) -> sgx_status_t {
    env_logger::init();
    info!("✔ [Enc] Running example inside enclave!");
    
    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    // A sample &'static string
    let rust_raw_string = "This is a in-Enclave ";
    // An array
    let word:[u8;4] = [82, 117, 115, 116];
    // An vector
    let word_vec:Vec<u8> = vec![32, 115, 116, 114, 105, 110, 103, 33];

    // Construct a string from &'static string
    let mut hello_string = String::from(rust_raw_string);

    // Iterate on word array
    for c in word.iter() {
        hello_string.push(*c as char);
    }

    // Rust style convertion
    hello_string += String::from_utf8(word_vec).expect("Invalid UTF-8")
                                               .as_str();

    // Ocall to normal world for output
    info!("###{}", &hello_string);

    // save to db
    let mut scratch_pad: Vec<u8> = vec![0; SCRATCH_PAD_SIZE];
    let scratch_pad_pointer: *mut u8 = &mut scratch_pad[0];

    let key: Vec<u8> = vec![6, 6, 6];
    save::save(key.clone(), hello_string, scratch_pad_pointer).and_then(
        |_| save::fetch(key.clone(), &mut scratch_pad)
    ).unwrap();

    sgx_status_t::SGX_SUCCESS
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct RandDataSerializable {
    key: u32,
    rand: [u8; 16],
    vec: Vec<u8>,
}

#[no_mangle]
pub extern "C" fn seal(blob: *mut u8, len: u32) -> sgx_status_t {

    let mut data = RandDataSerializable::default();
    data.key = 0x1234;

    let mut rand = match StdRng::new() {
        Ok(rng) => rng,
        Err(_) => { return sgx_status_t::SGX_ERROR_UNEXPECTED; },
    };
    rand.fill_bytes(&mut data.rand);

    data.vec.extend(data.rand.iter());

    let encoded_vec = serde_cbor::to_vec(&data).unwrap();
    let encoded_slice = encoded_vec.as_slice();
    info!("Length of encoded slice: {}", encoded_slice.len());
    info!("Encoded slice: {:?}", encoded_slice);

    let aad: [u8; 0] = [0_u8; 0];
    let result = SgxSealedData::<[u8]>::seal_data(&aad, encoded_slice);
    let sealed_data = match result {
        Ok(x) => x,
        Err(ret) => { return ret; },
    };

    let opt = to_sealed_log_for_slice(&sealed_data, blob, len);
    if opt.is_none() {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }

    info!("{:?}", data);

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn unseal(sealed_log: * mut u8, sealed_log_size: u32) -> sgx_status_t {

    let opt = from_sealed_log_for_slice::<u8>(sealed_log, sealed_log_size);
    let sealed_data = match opt {
        Some(x) => x,
        None => {
            return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
        },
    };

    let result = sealed_data.unseal_data();
    let unsealed_data = match result {
        Ok(x) => x,
        Err(ret) => {
            return ret;
        },
    };

    let encoded_slice = unsealed_data.get_decrypt_txt();
    info!("Length of encoded slice: {}", encoded_slice.len());
    info!("Encoded slice: {:?}", encoded_slice);
    let data: RandDataSerializable = serde_cbor::from_slice(encoded_slice).unwrap();

    info!("{:?}", data);

    sgx_status_t::SGX_SUCCESS
}

//////////////////////////////////////////////////////////////////////
/// utils
fn to_sealed_log_for_slice<T: Copy + ContiguousMemory>(sealed_data: &SgxSealedData<[T]>, sealed_log: * mut u8, sealed_log_size: u32) -> Option<* mut sgx_sealed_data_t> {
    unsafe {
        sealed_data.to_raw_sealed_data_t(sealed_log as * mut sgx_sealed_data_t, sealed_log_size)
    }
}

fn from_sealed_log_for_slice<'a, T: Copy + ContiguousMemory>(sealed_log: * mut u8, sealed_log_size: u32) -> Option<SgxSealedData<'a, [T]>> {
    unsafe {
        SgxSealedData::<[T]>::from_raw_sealed_data_t(sealed_log as * mut sgx_sealed_data_t, sealed_log_size)
    }
}

fn get_length_of_data_in_scratch_pad(scratch_pad: &Bytes) -> usize {
    let mut length_of_data_arr = [0u8; U32_NUM_BYTES];
    let bytes = &scratch_pad[..U32_NUM_BYTES];
    length_of_data_arr.copy_from_slice(bytes);
    u32::from_le_bytes(length_of_data_arr) as usize
}

fn get_data_from_scratch_pad(scratch_pad: &Bytes) -> Bytes {
    info!("✔ [Enc] get_data_from_scratch_pad!");

    let length_of_data = get_length_of_data_in_scratch_pad(scratch_pad);
    scratch_pad[U32_NUM_BYTES..U32_NUM_BYTES + length_of_data].to_vec()
}

//////////////////////////////////////////////////////////////////////////
/// Tests
#[no_mangle]
pub extern "C" fn test_main_entrance() -> size_t {
    rsgx_unit_tests!(
        test_seal_unseal,
    )
}