use crate::{
    common::{
        fuel_storage::Mappable,
        fuel_tx::{
            Receipt,
            Transaction,
            UtxoId,
        },
        fuel_types::{
            Bytes32,
            ContractId,
            MessageId,
        },
        fuel_vm::prelude::InterpreterError,
    },
    model::{
        Coin,
        FuelBlockConsensus,
        FuelBlockDb,
        Message,
    },
};
use std::io::ErrorKind;
use thiserror::Error;

pub use crate::common::fuel_vm::storage::{
    ContractsAssets,
    ContractsInfo,
    ContractsRawCode,
    ContractsState,
};

pub trait Transactional {
    /// Commits the pending state changes into the database.
    fn commit(self) -> Result<(), Error>;

    // TODO: It is work around for `Box<dyn DatabaseTransaction`>.
    //  with `DatabaseTransaction` from `fuel-core-storage` crate we can remove it.
    /// The same as `commit` but for case if the value is boxed.
    fn commit_box(self: Box<Self>) -> Result<(), Error>;
}

// TODO: Replace this trait with `Transaction<Database>` from `fuel-core-storage`.
//  `core::fmt::Debug` bound added here to be able to see a proper error in the tests without
//  updating them=)
/// The database transaction for the `Database` type.
pub trait DatabaseTransaction<Database>:
    Transactional + core::fmt::Debug + Send + Sync
{
    // TODO: After removing of `Box<dyn DatabaseTransaction>` replace this method with
    //  `AsRef<Database>` trait.
    /// Returns the reference to the `Database` instance.
    fn database(&self) -> &Database;
    // TODO: After removing of `Box<dyn DatabaseTransaction>` replace this method with
    //  `AsMut<Database>` trait.
    /// Returns the mutable reference to the `Database` instance.
    fn database_mut(&mut self) -> &mut Database;
}

/// The table of blocks generated by Fuels validators.
/// Right now, we have only that type of block, but we will support others in the future.
pub struct FuelBlocks;

impl Mappable for FuelBlocks {
    /// Unique identifier of the fuel block.
    type Key = Bytes32;
    type SetValue = FuelBlockDb;
    type GetValue = Self::SetValue;
}

/// The latest UTXO id of the contract. The contract's UTXO represents the unique id of the state.
/// After each transaction, old UTXO is consumed, and new UTXO is produced. UTXO is used as an
/// input to the next transaction related to the `ContractId` smart contract.
pub struct ContractsLatestUtxo;

impl Mappable for ContractsLatestUtxo {
    type Key = ContractId;
    /// The latest UTXO id.
    type SetValue = UtxoId;
    type GetValue = Self::SetValue;
}

/// Receipts of different hidden internal operations.
pub struct Receipts;

impl Mappable for Receipts {
    /// Unique identifier of the transaction.
    type Key = Bytes32;
    type SetValue = [Receipt];
    type GetValue = Vec<Receipt>;
}

/// The table of consensus metadata associated with sealed (finalized) blocks
pub struct SealedBlockConsensus;

impl Mappable for SealedBlockConsensus {
    type Key = Bytes32;
    type SetValue = FuelBlockConsensus;
    type GetValue = Self::SetValue;
}

/// The storage table of coins. Each [`Coin`](crate::model::Coin) is represented by unique `UtxoId`.
pub struct Coins;

impl Mappable for Coins {
    type Key = UtxoId;
    type SetValue = Coin;
    type GetValue = Self::SetValue;
}

/// The storage table of bridged Ethereum [`Message`](crate::model::Message)s.
pub struct Messages;

impl Mappable for Messages {
    type Key = MessageId;
    type SetValue = Message;
    type GetValue = Self::SetValue;
}

/// The storage table of confirmed transactions.
pub struct Transactions;

impl Mappable for Transactions {
    type Key = Bytes32;
    type SetValue = Transaction;
    type GetValue = Self::SetValue;
}

// TODO: Add macro to define all common tables to avoid copy/paste of the code.
// TODO: Add macro to define common unit tests.

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("error performing binary serialization")]
    Codec,
    #[error("Failed to initialize chain")]
    ChainAlreadyInitialized,
    #[error("Chain is not yet initialized")]
    ChainUninitialized,
    #[error("Invalid database version")]
    InvalidDatabaseVersion,
    #[error("error occurred in the underlying datastore `{0}`")]
    DatabaseError(Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<Error> for std::io::Error {
    fn from(e: Error) -> Self {
        std::io::Error::new(ErrorKind::Other, e)
    }
}

#[derive(Debug, Error)]
pub enum KvStoreError {
    #[error("generic error occurred")]
    Error(Box<dyn std::error::Error + Send + Sync>),
    /// This error should be created with `not_found` macro.
    #[error("resource of type `{0}` was not found at the: {1}")]
    NotFound(&'static str, &'static str),
}

/// Creates `KvStoreError::NotFound` error with file and line information inside.
///
/// # Examples
///
/// ```
/// use fuel_core_interfaces::not_found;
/// use fuel_core_interfaces::db::Messages;
///
/// let string_type = not_found!("BlockId");
/// let mappable_type = not_found!(Messages);
/// let mappable_path = not_found!(fuel_core_interfaces::db::Coins);
/// ```
#[macro_export]
macro_rules! not_found {
    ($name: literal) => {
        $crate::db::KvStoreError::NotFound($name, concat!(file!(), ":", line!()))
    };
    ($ty: path) => {
        $crate::db::KvStoreError::NotFound(
            ::core::any::type_name::<
                <$ty as $crate::common::fuel_storage::Mappable>::GetValue,
            >(),
            concat!(file!(), ":", line!()),
        )
    };
}

impl From<Error> for KvStoreError {
    fn from(e: Error) -> Self {
        KvStoreError::Error(Box::new(e))
    }
}

impl From<KvStoreError> for Error {
    fn from(e: KvStoreError) -> Self {
        Error::DatabaseError(Box::new(e))
    }
}

impl From<KvStoreError> for std::io::Error {
    fn from(e: KvStoreError) -> Self {
        std::io::Error::new(ErrorKind::Other, e)
    }
}

impl From<Error> for InterpreterError {
    fn from(e: Error) -> Self {
        InterpreterError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl From<KvStoreError> for InterpreterError {
    fn from(e: KvStoreError) -> Self {
        InterpreterError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn not_found_output() {
        #[rustfmt::skip]
        assert_eq!(
            format!("{}", not_found!("BlockId")),
            format!("resource of type `BlockId` was not found at the: {}:{}", file!(), line!() - 1)
        );
        #[rustfmt::skip]
        assert_eq!(
            format!("{}", not_found!(Coins)),
            format!("resource of type `fuel_core_interfaces::model::coin::Coin` was not found at the: {}:{}", file!(), line!() - 1)
        );
    }
}