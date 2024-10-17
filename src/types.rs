use anyhow::Result;

use std::convert::TryFrom;

use satsnet::blockdata::block::Header as BlockHeader;
use satsnet::{
    consensus::encode::{deserialize, Decodable, Encodable},
    hashes::{hash_newtype, sha256, Hash},
    io, OutPoint, Script, Txid,
};
use satsnet_slices::bsl;

macro_rules! impl_consensus_encoding {
    ($thing:ident, $($field:ident),+) => (
        impl Encodable for $thing {
            #[inline]
            fn consensus_encode<S: io::Write + ?Sized>(
                &self,
                s: &mut S,
            ) -> Result<usize, io::Error> {
                let mut len = 0;
                $(len += self.$field.consensus_encode(s)?;)+
                Ok(len)
            }
        }

        impl Decodable for $thing {
            #[inline]
            fn consensus_decode<D: io::BufRead + ?Sized>(
                d: &mut D,
            ) -> Result<$thing, satsnet::consensus::encode::Error> {
                Ok($thing {
                    $($field: Decodable::consensus_decode(d)?),+
                })
            }
        }
    );
}

pub const HASH_PREFIX_LEN: usize = 8;
const HEIGHT_SIZE: usize = 4;

pub(crate) type HashPrefix = [u8; HASH_PREFIX_LEN];
pub(crate) type SerializedHashPrefixRow = [u8; HASH_PREFIX_ROW_SIZE];
type Height = u32;
pub(crate) type SerBlock = Vec<u8>;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct HashPrefixRow {
    prefix: HashPrefix,
    height: Height, // transaction confirmed height
}

pub const HASH_PREFIX_ROW_SIZE: usize = HASH_PREFIX_LEN + HEIGHT_SIZE;

impl HashPrefixRow {
    pub(crate) fn to_db_row(&self) -> SerializedHashPrefixRow {
        let mut row = [0; HASH_PREFIX_ROW_SIZE];
        let len = self
            .consensus_encode(&mut (&mut row as &mut [u8]))
            .expect("in-memory writers don't error");
        debug_assert_eq!(len, HASH_PREFIX_ROW_SIZE);
        row
    }

    pub(crate) fn from_db_row(row: SerializedHashPrefixRow) -> Self {
        deserialize(&row).expect("bad HashPrefixRow")
    }

    pub fn height(&self) -> usize {
        usize::try_from(self.height).expect("invalid height")
    }
}

impl_consensus_encoding!(HashPrefixRow, prefix, height);

hash_newtype! {
    /// https://electrumx-spesmilo.readthedocs.io/en/latest/protocol-basics.html#script-hashes
    #[hash_newtype(backward)]
    pub struct ScriptHash(sha256::Hash);
}

impl ScriptHash {
    pub fn new(script: &Script) -> Self {
        ScriptHash::hash(script.as_bytes())
    }

    fn prefix(&self) -> HashPrefix {
        let mut prefix = HashPrefix::default();
        prefix.copy_from_slice(&self.0[..HASH_PREFIX_LEN]);
        prefix
    }
}

pub(crate) struct ScriptHashRow;

impl ScriptHashRow {
    pub(crate) fn scan_prefix(scripthash: ScriptHash) -> HashPrefix {
        scripthash.0[..HASH_PREFIX_LEN].try_into().unwrap()
    }

    pub(crate) fn row(scripthash: ScriptHash, height: usize) -> HashPrefixRow {
        HashPrefixRow {
            prefix: scripthash.prefix(),
            height: Height::try_from(height).expect("invalid height"),
        }
    }
}

// ***************************************************************************

hash_newtype! {
    /// https://electrumx-spesmilo.readthedocs.io/en/latest/protocol-basics.html#status
    pub struct StatusHash(sha256::Hash);
}

// ***************************************************************************

fn spending_prefix(prev: OutPoint) -> HashPrefix {
    let txid_prefix = HashPrefix::try_from(&prev.txid[..HASH_PREFIX_LEN]).unwrap();
    let value = u64::from_be_bytes(txid_prefix);
    let value = value.wrapping_add(prev.vout.into());
    value.to_be_bytes()
}

pub(crate) struct SpendingPrefixRow;

impl SpendingPrefixRow {
    pub(crate) fn scan_prefix(outpoint: OutPoint) -> HashPrefix {
        spending_prefix(outpoint)
    }

    pub(crate) fn row(outpoint: OutPoint, height: usize) -> HashPrefixRow {
        HashPrefixRow {
            prefix: spending_prefix(outpoint),
            height: Height::try_from(height).expect("invalid height"),
        }
    }
}

// ***************************************************************************

fn txid_prefix(txid: &Txid) -> HashPrefix {
    let mut prefix = [0u8; HASH_PREFIX_LEN];
    prefix.copy_from_slice(&txid[..HASH_PREFIX_LEN]);
    prefix
}

pub(crate) struct TxidRow;

impl TxidRow {
    pub(crate) fn scan_prefix(txid: Txid) -> HashPrefix {
        txid_prefix(&txid)
    }

    pub(crate) fn row(txid: Txid, height: usize) -> HashPrefixRow {
        HashPrefixRow {
            prefix: txid_prefix(&txid),
            height: Height::try_from(height).expect("invalid height"),
        }
    }
}

// ***************************************************************************

pub(crate) type SerializedHeaderRow = [u8; HEADER_ROW_SIZE];

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct HeaderRow {
    pub(crate) header: BlockHeader,
}

pub const HEADER_ROW_SIZE: usize = 80;

impl_consensus_encoding!(HeaderRow, header);

impl HeaderRow {
    pub(crate) fn new(header: BlockHeader) -> Self {
        Self { header }
    }

    pub(crate) fn to_db_row(&self) -> SerializedHeaderRow {
        let mut row = [0; HEADER_ROW_SIZE];
        let len = self
            .consensus_encode(&mut (&mut row as &mut [u8]))
            .expect("in-memory writers don't error");
        debug_assert_eq!(len, HEADER_ROW_SIZE);
        row
    }

    pub(crate) fn from_db_row(row: SerializedHeaderRow) -> Self {
        deserialize(&row).expect("bad HeaderRow")
    }
}

pub(crate) fn bsl_txid(tx: &bsl::Transaction) -> Txid {
    satsnet::Txid::from_slice(tx.txid_sha2().as_slice()).expect("invalid txid")
}