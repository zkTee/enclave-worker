use sgx_types::*;
use sgx_tseal::SgxSealedData;
use {to_sealed_log_for_slice, from_sealed_log_for_slice, get_data_from_scratch_pad};
use Item;

use ocall_api::{
    save_to_db,
    get_from_db,
};

use std::{
    u32,
    vec::Vec,
    mem::size_of,
    string::{
        String,
        ToString,
    },
};
use log::*;
use SCRATCH_PAD_SIZE;

pub fn save(  
    mut key: Vec<u8>,
    value: String,
    scratch_pad_pointer: *mut u8,
) -> Result<sgx_status_t, String> {
    info!("✔ [Enc] Sealing data...");
    let data = Item::new(value.clone());
    info!("✔ [Enc] Value to seal: {:?}", data.name);

    let encoded_data = serde_cbor::to_vec(&data).unwrap();
    let encoded_slice = encoded_data.as_slice();
    let extra_data: [u8; 0] = [0u8; 0]; // TODO Abstract this away!
    let sealing_result = SgxSealedData::<[u8]>::seal_data(
        &extra_data,
        encoded_slice,
    );
    let sealed_data = match sealing_result {
        Ok(sealed_data) => sealed_data,
        Err(sgx_error) => return Err(sgx_error.to_string())
    };
    trace!(
        "✔ [Enc] Sealed-data additional data: {:?}",
        sealed_data.get_additional_txt()
    );
    trace!(
        "✔ [Enc] Sealed-data encrypted txt: {:?}",
        sealed_data.get_encrypt_txt()
    );
    trace!(
        "✔ [Enc] Sealed-data payload size: {:?}",
        sealed_data.get_payload_size()
    );
    trace!("✔ [Enc] Raw sealed data size: {:?}",
        SgxSealedData::<u8>::calc_raw_sealed_data_size(
            sealed_data.get_add_mac_txt_len(),
            sealed_data.get_encrypt_txt_len(),
        )
    );
    trace!("✔ [Enc] Data sealed successfully!");

    let sealed_log_size = size_of::<sgx_sealed_data_t>() + encoded_slice.len();
    trace!("✔ [Enc] Sealed log size: {}", sealed_log_size);
    let option = to_sealed_log_for_slice(
        &sealed_data,
        scratch_pad_pointer,
        sealed_log_size as u32,
    );
    if option.is_none() {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER.to_string())
    }

    info!("✔ [Enc] Sealed data written into app's scratch-pad!");
    info!("✔ [Enc] Sending db key & sealed data size via OCALL...");
    
    let key_pointer: *mut u8 = &mut key[0];
    unsafe {
        save_to_db(
            &mut sgx_status_t::SGX_SUCCESS,
            key_pointer,
            key.len() as *const u32,
            sealed_log_size as *const u32,
            scratch_pad_pointer,
        )
    };
    Ok(sgx_status_t::SGX_SUCCESS)
}

pub fn fetch(
    mut key: Vec<u8>,
    scratch_pad: &mut Vec<u8>,
) -> Result<sgx_status_t, String> {
    info!("✔ [Enc] Getting item from external db...");
    let key_pointer: *mut u8 = &mut key[0];
    let enclave_scratch_pad_pointer: *mut u8 = &mut scratch_pad[0];
    let status = unsafe {
        get_from_db(
            &mut sgx_status_t::SGX_SUCCESS,
            key_pointer,
            key.len() as *const u32,
            enclave_scratch_pad_pointer,
            SCRATCH_PAD_SIZE as *const u32,
        )
    };
    println!("✔ [Enc] get from db status: {:?}", status);

    let mut data = get_data_from_scratch_pad(&scratch_pad);
    info!("✔ [Enc] External data written to enclave's scratch pad!");
    trace!("✔ [Enc] Retreived data length: {:?}", data.len());

    let data_pointer: *mut u8 = &mut data[0];
    let maybe_sealed_data = from_sealed_log_for_slice::<u8>(
        data_pointer,
        data.len() as u32
    );
    let sealed_data = match maybe_sealed_data {
        Some(sealed_data) => sealed_data,
        None => return Err(
            sgx_status_t::SGX_ERROR_INVALID_PARAMETER.to_string()
        )
    };
    trace!(
        "✔ [Enc] Payload: {:?}",
        sealed_data.get_payload_size()
    );
    trace!(
        "✔ [Enc] Encrypted text: {:?}",
        sealed_data.get_encrypt_txt()
    );
    trace!(
        "✔ [Enc] Additional text: {:?}",
        sealed_data.get_additional_txt()
    );
    let unsealed_data = match sealed_data.unseal_data() {
        Ok(unsealed_data) => unsealed_data,
        Err(e) => return Err(e.to_string())
    };
    let cbor_encoded_slice = unsealed_data.get_decrypt_txt();
    let final_data: Item = serde_cbor::from_slice(
        cbor_encoded_slice
    ).unwrap();
    //info!("✔ [Enc] Final unsealed data: {:?}", final_data);
    info!("✔ [Enc] Final unsealed name: {:?}", final_data.name);
    Ok(sgx_status_t::SGX_SUCCESS)
}
