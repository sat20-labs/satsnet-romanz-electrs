use std::collections::HashMap;

use bitcoin::blockdata::block::Header as BlockHeader;
use bitcoin::{BlockHash, Network, CompactTarget};
use bitcoin::{hashes::Hash, TxMerkleNode};

/// A new header found, to be added to the chain at specific height
pub(crate) struct NewHeader {
    header: BlockHeader,
    hash: BlockHash,
    height: usize,
}

impl NewHeader {
    pub(crate) fn from((header, height): (BlockHeader, usize)) -> Self {
        Self {
            header,
            hash: header.block_hash(),
            height,
        }
    }

    pub(crate) fn height(&self) -> usize {
        self.height
    }

    pub(crate) fn hash(&self) -> BlockHash {
        self.hash
    }
}

/// Current blockchain headers' list
pub struct Chain {
    headers: Vec<(BlockHash, BlockHeader)>,
    heights: HashMap<BlockHash, usize>,
}

impl Chain {
    // create an empty chain
    // pub fn new(network: Network) -> Self {
    //     let genesis = bitcoin::blockdata::constants::genesis_block(network);
    //     let genesis_hash = genesis.block_hash();
    //     info!("fact genesis hash: {}", genesis_hash);
    //     Self {
    //         headers: vec![(genesis_hash, genesis.header)],
    //         heights: std::iter::once((genesis_hash, 0)).collect(), // genesis header @ zero height
    //     }
    // }
    pub fn new(network: Network) -> Self {
        // 定义 mainnet genesis
        let (genesis_hash, genesis_header) = match network {
            Network::Satsnet => {
                let hash = BlockHash::from_byte_array([
                    0x8b, 0xb2, 0x8a, 0x29, 0xfe, 0xb3, 0x8b, 0x85,
                    0xb1, 0x7f, 0x16, 0x42, 0x46, 0xf0, 0xb7, 0x9f,
                    0xdb, 0x79, 0xf5, 0x22, 0xf4, 0xb6, 0x9a, 0x87,
                    0xee, 0x74, 0xa0, 0x52, 0x71, 0x63, 0x61, 0x41,
                ]);

                let merkle_root = TxMerkleNode::from_byte_array([
                    0x22, 0x89, 0xb6, 0xf8, 0xc9, 0x0d, 0xce, 0x99,
                    0xf8, 0x36, 0x8a, 0xe3, 0x93, 0x81, 0x2b, 0xcc,
                    0x0a, 0xa4, 0xa6, 0xb1, 0xab, 0xa5, 0xf3, 0xb2,
                    0x0c, 0x1a, 0x71, 0xbc, 0xe8, 0xdd, 0x18, 0x9e,
                ]);

                let header = BlockHeader {
                    version: bitcoin::block::Version::ONE,
                    prev_blockhash: BlockHash::all_zeros(),
                    merkle_root,
                    time: 1751454131,
                    bits: CompactTarget::from_consensus(0),
                    nonce: 2466277953,
                };
                (hash, header)
            }

            Network::Satstestnet => {
                let hash = BlockHash::from_byte_array([
                    0x21, 0x3a, 0xb6, 0xeb, 0x99, 0x39, 0x1c, 0xbb,
                    0xb2, 0xb9, 0x8c, 0xaf, 0x93, 0x47, 0xaa, 0xb5,
                    0xf4, 0x72, 0xd6, 0x95, 0x1f, 0xe2, 0x66, 0x70,
                    0x57, 0xe1, 0x43, 0x4e, 0x04, 0x68, 0x1b, 0xdf,
                ]);

                let merkle_root = TxMerkleNode::from_byte_array([
                    0xe8, 0xf5, 0x87, 0xe6, 0xd2, 0x00, 0xca, 0x9c,
                    0x03, 0x92, 0xee, 0x74, 0x6d, 0xab, 0x24, 0xfd,
                    0x7e, 0xf8, 0x01, 0x7b, 0xc2, 0xf4, 0x8d, 0x07,
                    0x21, 0xe8, 0x78, 0x45, 0x41, 0x27, 0x55, 0xfa,
                ]);

                let header = BlockHeader {
                    version: bitcoin::block::Version::ONE,
                    prev_blockhash: BlockHash::all_zeros(),
                    merkle_root,
                    time: 1733136112,
                    bits: CompactTarget::from_consensus(0),
                    nonce: 1182242621,
                };
                (hash, header)
            }

            // 其它网络默认用原来的
            other => {
                let genesis = bitcoin::blockdata::constants::genesis_block(other);
                (genesis.block_hash(), genesis.header)
            }
        };

        Self {
            headers: vec![(genesis_hash, genesis_header)],
            heights: std::iter::once((genesis_hash, 0)).collect(),
        }
    }

    pub(crate) fn drop_last_headers(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        let new_height = self.height().saturating_sub(n);
        self.update(vec![NewHeader::from((
            self.headers[new_height].1,
            new_height,
        ))])
    }

    /// Load the chain from a collection of headers, up to the given tip
    pub(crate) fn load(&mut self, headers: impl Iterator<Item = BlockHeader>, tip: BlockHash) {
        let genesis_hash = self.headers[0].0;

        // 22e46d9d7a8e0b648555a084892bd8c0637b931b2ad029b01fe55f851df90dd3
        // let genesis_hash_str = "df1b68044e43e1577066e21f95d672f4b5aa4793af8cb9b2bb1c3999ebb63a21";
        info!("fact genesis hash: {}", genesis_hash);
        
        let header_map: HashMap<BlockHash, BlockHeader> =
            headers.map(|h| (h.block_hash(), h)).collect();
        let mut blockhash = tip;
        // let mut blockhash_str = blockhash.to_string();
        let mut new_headers: Vec<&BlockHeader> = Vec::with_capacity(header_map.len());
        // while blockhash_str != genesis_hash_str {
        while blockhash != genesis_hash {
            let header = match header_map.get(&blockhash) {
                Some(header) => header,
                None => panic!("missing header {} while loading from DB", blockhash),
            };
            blockhash = header.prev_blockhash;
            // blockhash_str = blockhash.to_string();
            new_headers.push(header);
        }
        info!("loading {} headers, tip={}", new_headers.len(), tip);
        let new_headers = new_headers.into_iter().rev().copied(); // order by height
        self.update(new_headers.zip(1..).map(NewHeader::from).collect())
    }

    /// Get the block hash at specified height (if exists)
    pub(crate) fn get_block_hash(&self, height: usize) -> Option<BlockHash> {
        self.headers.get(height).map(|(hash, _header)| *hash)
    }

    /// Get the block header at specified height (if exists)
    pub(crate) fn get_block_header(&self, height: usize) -> Option<&BlockHeader> {
        self.headers.get(height).map(|(_hash, header)| header)
    }

    /// Get the block height given the specified hash (if exists)
    pub(crate) fn get_block_height(&self, blockhash: &BlockHash) -> Option<usize> {
        self.heights.get(blockhash).copied()
    }

    /// Update the chain with a list of new headers (possibly a reorg)
    pub(crate) fn update(&mut self, headers: Vec<NewHeader>) {
        if let Some(first_height) = headers.first().map(|h| h.height) {
            for (hash, _header) in self.headers.drain(first_height..) {
                assert!(self.heights.remove(&hash).is_some());
            }
            for (h, height) in headers.into_iter().zip(first_height..) {
                assert_eq!(h.height, height);
                assert_eq!(h.hash, h.header.block_hash());
                assert!(self.heights.insert(h.hash, h.height).is_none());
                self.headers.push((h.hash, h.header));
            }
            info!(
                "chain updated: tip={}, height={}",
                self.headers.last().unwrap().0,
                self.headers.len() - 1
            );
        }
    }

    /// Best block hash
    pub(crate) fn tip(&self) -> BlockHash {
        self.headers.last().expect("empty chain").0
    }

    /// Number of blocks (excluding genesis block)
    pub(crate) fn height(&self) -> usize {
        self.headers.len() - 1
    }

    /// List of block hashes for efficient fork detection and block/header sync
    /// see https://en.bitcoin.it/wiki/Protocol_documentation#getblocks
    pub(crate) fn locator(&self) -> Vec<BlockHash> {
        let mut result = vec![];
        let mut index = self.headers.len() - 1;
        let mut step = 1;
        loop {
            if result.len() >= 10 {
                step *= 2;
            }
            result.push(self.headers[index].0);
            if index == 0 {
                break;
            }
            index = index.saturating_sub(step);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{Chain, NewHeader};
    use bitcoin::blockdata::block::Header as BlockHeader;
    use bitcoin::consensus::deserialize;
    use bitcoin::Network::Regtest;
    use hex_lit::hex;

    #[test]
    fn test_genesis() {
        let regtest = Chain::new(Regtest);
        assert_eq!(regtest.height(), 0);
        assert_eq!(
            regtest.tip(),
            "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206"
                .parse()
                .unwrap()
        );
    }

    #[test]
    fn test_updates() {
        let byte_headers = [
hex!("0000002006226e46111a0b59caaf126043eb5bbf28c34f3a5e332a1fc7b2b73cf188910f1d14d3c7ff12d6adf494ebbcfba69baa915a066358b68a2b8c37126f74de396b1d61cc60ffff7f2000000000"),
hex!("00000020d700ae5d3c705702e0a5d9ababd22ded079f8a63b880b1866321d6bfcb028c3fc816efcf0e84ccafa1dda26be337f58d41b438170c357cda33a68af5550590bc1e61cc60ffff7f2004000000"),
hex!("00000020d13731bc59bc0989e06a5e7cab9843a4e17ad65c7ca47cd77f50dfd24f1f55793f7f342526aca9adb6ce8f33d8a07662c97d29d83b9e18117fb3eceecb2ab99b1e61cc60ffff7f2001000000"),
hex!("00000020a603def3e1255cadfb6df072946327c58b344f9bfb133e8e3e280d1c2d55b31c731a68f70219472864a7cb010cd53dc7e0f67e57f7d08b97e5e092b0c3942ad51f61cc60ffff7f2001000000"),
hex!("0000002041dd202b3b2edcdd3c8582117376347d48ff79ff97c95e5ac814820462012e785142dc360975b982ca43eecd14b4ba6f019041819d4fc5936255d7a2c45a96651f61cc60ffff7f2000000000"),
hex!("0000002072e297a2d6b633c44f3c9b1a340d06f3ce4e6bcd79ebd4c4ff1c249a77e1e37c59c7be1ca0964452e1735c0d2740f0d98a11445a6140c36b55770b5c0bcf801f1f61cc60ffff7f2000000000"),
hex!("000000200c9eb5889a8e924d1c4e8e79a716514579e41114ef37d72295df8869d6718e4ac5840f28de43ff25c7b9200aaf7873b20587c92827eaa61943484ca828bdd2e11f61cc60ffff7f2000000000"),
hex!("000000205873f322b333933e656b07881bb399dae61a6c0fa74188b5fb0e3dd71c9e2442f9e2f433f54466900407cf6a9f676913dd54aad977f7b05afcd6dcd81e98ee752061cc60ffff7f2004000000"),
hex!("00000020fd1120713506267f1dba2e1856ca1d4490077d261cde8d3e182677880df0d856bf94cfa5e189c85462813751ab4059643759ed319a81e0617113758f8adf67bc2061cc60ffff7f2000000000"),
hex!("000000200030d7f9c11ef35b89a0eefb9a5e449909339b5e7854d99804ea8d6a49bf900a0304d2e55fe0b6415949cff9bca0f88c0717884a5e5797509f89f856af93624a2061cc60ffff7f2002000000"),
        ];
        let headers: Vec<BlockHeader> = byte_headers
            .iter()
            .map(|byte_header| deserialize(byte_header).unwrap())
            .collect();

        for chunk_size in 1..headers.len() {
            let mut regtest = Chain::new(Regtest);
            let mut height = 0;
            let mut tip = regtest.tip();
            for chunk in headers.chunks(chunk_size) {
                let mut update = vec![];
                for header in chunk {
                    height += 1;
                    tip = header.block_hash();
                    update.push(NewHeader::from((*header, height)))
                }
                regtest.update(update);
                assert_eq!(regtest.tip(), tip);
                assert_eq!(regtest.height(), height);
            }
            assert_eq!(regtest.tip(), headers.last().unwrap().block_hash());
            assert_eq!(regtest.height(), headers.len());
        }

        // test loading from a list of headers and tip
        let mut regtest = Chain::new(Regtest);
        regtest.load(
            headers.iter().copied(),
            headers.last().unwrap().block_hash(),
        );
        assert_eq!(regtest.height(), headers.len());

        // test getters
        for (header, height) in headers.iter().zip(1usize..) {
            assert_eq!(regtest.get_block_header(height), Some(header));
            assert_eq!(regtest.get_block_hash(height), Some(header.block_hash()));
            assert_eq!(regtest.get_block_height(&header.block_hash()), Some(height));
        }

        // test chain shortening
        for i in (0..=headers.len()).rev() {
            let hash = regtest.get_block_hash(i).unwrap();
            assert_eq!(regtest.get_block_height(&hash), Some(i));
            assert_eq!(regtest.height(), i);
            assert_eq!(regtest.tip(), hash);
            regtest.drop_last_headers(1);
        }
        assert_eq!(regtest.height(), 0);
        assert_eq!(
            regtest.tip(),
            "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206"
                .parse()
                .unwrap()
        );

        regtest.drop_last_headers(1);
        assert_eq!(regtest.height(), 0);
        assert_eq!(
            regtest.tip(),
            "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206"
                .parse()
                .unwrap()
        );

        // test reorg
        let mut regtest = Chain::new(Regtest);
        regtest.load(
            headers.iter().copied(),
            headers.last().unwrap().block_hash(),
        );
        let height = regtest.height();

        let new_header: BlockHeader = deserialize(&hex!("000000200030d7f9c11ef35b89a0eefb9a5e449909339b5e7854d99804ea8d6a49bf900a0304d2e55fe0b6415949cff9bca0f88c0717884a5e5797509f89f856af93624a7a6bcc60ffff7f2000000000")).unwrap();
        regtest.update(vec![NewHeader::from((new_header, height))]);
        assert_eq!(regtest.height(), height);
        assert_eq!(
            regtest.tip(),
            "0e16637fe0700a7c52e9a6eaa58bd6ac7202652103be8f778680c66f51ad2e9b"
                .parse()
                .unwrap()
        );
    }
}
