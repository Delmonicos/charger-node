//! RPC interface for the transaction payment module.

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
pub use session_payment_runtime_api::SessionPaymentApi as SessionPaymentRuntimeApi;

#[rpc]
pub trait SessionPaymentApi<BlockHash> {
	#[rpc(name = "sessionPayment_getNbAllowed")]
	fn get_nb_allowed(&self, at: Option<BlockHash>) -> Result<u32>;
}

/// A struct that implements the `SessionPaymentApi`.
pub struct SessionPayment<C, M> {

	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> SessionPayment<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		SessionPayment {
			client,
			_marker: Default::default(),
		}
	}
}

/// Error type of this RPC api.
// pub enum Error {
// 	/// The transaction was not decodable.
// 	DecodeError,
// 	/// The call to runtime failed.
// 	RuntimeError,
// }
//
// impl From<Error> for i64 {
// 	fn from(e: Error) -> i64 {
// 		match e {
// 			Error::RuntimeError => 1,
// 			Error::DecodeError => 2,
// 		}
// 	}
// }


impl<C, Block> SessionPaymentApi<<Block as BlockT>::Hash> for SessionPayment<C, Block>
	where
		Block: BlockT,
		C: Send + Sync + 'static,
		C: ProvideRuntimeApi<Block>,
		C: HeaderBackend<Block>,
		C::Api: SessionPaymentRuntimeApi<Block>,
{
	fn get_nb_allowed(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u32> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.get_nb_allowed(&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
