use sgx_types::*;
use init;
use seal;
use say;

pub fn main() {
    let enclave = match init::init_enclave() {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        }
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        }
    };

    let status = say::say_something(&enclave);

    // let result = seal::seal(&enclave);
    match status {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", status.as_str());
            return;
        }
    }

    enclave.destroy();
}