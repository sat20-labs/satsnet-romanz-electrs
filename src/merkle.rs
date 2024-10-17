use satsnet::{hash_types::TxMerkleNode, hashes::Hash, Txid};

pub(crate) struct Proof {
    proof: Vec<TxMerkleNode>,
    position: usize,
}

impl Proof {
    pub(crate) fn create(txids: &[Txid], position: usize) -> Self {
        assert!(position < txids.len());
        let mut offset = position;
        let mut hashes: Vec<TxMerkleNode> = txids
            .iter()
            .map(|txid| TxMerkleNode::from_raw_hash(txid.to_raw_hash()))
            .collect();

        let mut proof = vec![];
        while hashes.len() > 1 {
            if hashes.len() % 2 != 0 {
                let last = *hashes.last().unwrap();
                hashes.push(last);
            }
            offset = if offset % 2 == 0 {
                offset + 1
            } else {
                offset - 1
            };
            proof.push(hashes[offset]);
            offset /= 2;
            hashes = hashes
                .chunks(2)
                .map(|pair| {
                    let left = pair[0];
                    let right = pair[1];
                    let input = [&left[..], &right[..]].concat();
                    TxMerkleNode::hash(&input)
                })
                .collect()
        }
        Self { proof, position }
    }

    pub(crate) fn to_hex(&self) -> Vec<String> {
        self.proof
            .iter()
            .map(|node| format!("{:x}", node))
            .collect()
    }

    pub(crate) fn position(&self) -> usize {
        self.position
    }
}

