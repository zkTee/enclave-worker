use sgx_types::*;

extern "C" {
    pub fn seal(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        blob: *mut u8,
        len: u32,
    ) -> sgx_status_t;
    
    pub fn unseal(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        blob: *mut u8,
        len: u32,
    ) -> sgx_status_t;

    pub fn test_main_entrance(eid: sgx_enclave_id_t, retval: *mut size_t) -> sgx_status_t;
}