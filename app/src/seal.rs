use sgx_types::*;
use ecall_api;
use sgx_urts::SgxEnclave;

pub fn seal(enclave: &SgxEnclave) -> sgx_status_t {
    let input_string = String::from("I am confidential.");
    let len = input_string.len() as u32;
    let mut retval = sgx_status_t::SGX_SUCCESS;

    let status = unsafe {
        ecall_api::seal(
            enclave.geteid(),
            &mut retval,
            input_string.as_ptr() as *mut u8,
            len,
        )
    };

    status
}