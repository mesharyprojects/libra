address 0x1 {

module Signature {
    native public fun ed25519_validate_pubkey(public_key: vector<u8>): bool;
    native public fun ed25519_verify(signature: vector<u8>, public_key: vector<u8>, message: vector<u8>): bool;
    native public fun ed25519_threshold_verify(bitmap: vector<u8>, signature: vector<u8>, public_key: vector<u8>, message: vector<u8>): u64;
}

}
