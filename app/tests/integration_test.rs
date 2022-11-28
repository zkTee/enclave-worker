extern crate sgx_types;
use sgx_types::*;
mod mock_enclave;
mod ffi;

#[test]
pub fn enclave_works() {
    let enclave = mock_enclave::init_enclave().unwrap();

    let mut retval = 0usize;

    let result = unsafe { ffi::test_main_entrance(enclave.geteid(), &mut retval) };

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