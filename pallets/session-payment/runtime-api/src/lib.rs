#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::vec::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait SessionPaymentApi {
		fn get_nb_allowed() -> u32;
		fn get_payment_consents() -> Vec<(Vec<u8>, Vec<u8>)>;
	}
}
