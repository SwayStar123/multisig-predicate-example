predicate;

use std::tx::{
    tx_witness_data,
    tx_witnesses_count,
    tx_id
};
use std::b512::B512;
use std::ecr::ec_recover_address;

configurable {
    ADDRESS_ONE: b256 = 0x9b032378bc702c27c530dada023f514ee1dbaccf0039d176005a1027753a5e72,
    ADDRESS_TWO: b256 = 0xe10f526b192593793b7a1559a391445faba82a1d669e3eb2dcd17f9c121b24b1,
    ADDRESS_THREE: b256 = 0xe10f526b192593793b7a1559a391445faba82a1d669e3eb2dcd17f9c121b24b1,

    REQUIRED_SIGNATURES: u64 = 2,
}

fn main() -> bool {    
    let key_addresses = [
        ADDRESS_ONE,
        ADDRESS_TWO,
        ADDRESS_THREE
    ]; 

    // Check all signatures are unique.
    let mut counts = [0, 0, 0];
    let mut valid_sigs = 0;
    let mut tx_signature_count = tx_witnesses_count();
    let msg_hash: b256 = tx_id();

    while tx_signature_count != 0 {
        let signature: B512 = tx_witness_data(tx_signature_count);
        let curr_address_signature = ec_recover_address(signature, msg_hash).unwrap();

        let mut i = 0;
        while i < 3 {
            if curr_address_signature.value == key_addresses[i] {
                counts[i] += 1;
                valid_sigs += 1;
            }
            i += 1;
        }
    }

    if counts[0] <= 1 && counts[1] <= 1 && counts[2] <= 1 {
        if valid_sigs >= REQUIRED_SIGNATURES {
            return true;
        }
    }

    return false;
}