use bitcoin::blockdata::block::BlockHeader;
use bitcoin::network::encodable::ConsensusDecodable;
use bitcoin::network::serialize::BitcoinHash;
use bitcoin::network::serialize::deserialize;
use bitcoin::network::serialize::RawDecoder;
use itertools::enumerate;
use reqwest;
use serde_json::{from_slice, Value};
use std::collections::VecDeque;
use std::io::Cursor;
use std::iter::FromIterator;

use types::{Bytes, HeaderMap, Sha256dHash};

const HEADER_SIZE: usize = 80;

pub struct Daemon {
    url: String,
}

pub struct HeaderList {
    headers: Vec<(usize, BlockHeader)>,
}

impl HeaderList {
    pub fn best_header(&self) -> &BlockHeader {
        &self.headers.last().unwrap().1
    }

    pub fn headers(&self) -> &[(usize, BlockHeader)] {
        &self.headers
    }

    pub fn into_map(self) -> HeaderMap {
        HeaderMap::from_iter(
            self.headers
                .into_iter()
                .map(|item| (item.1.bitcoin_hash(), item.1)),
        )
    }
}

impl Daemon {
    pub fn new(url: &str) -> Daemon {
        Daemon {
            url: url.to_string(),
        }
    }

    fn request(&self, resource: &str) -> reqwest::Response {
        let url = format!("{}/rest/{}", self.url, resource);
        reqwest::get(&url).unwrap()
    }

    pub fn get(&self, resource: &str) -> Bytes {
        let mut buf = Bytes::new();
        let mut resp = self.request(resource);
        resp.copy_to(&mut buf).unwrap();
        buf
    }

    fn get_all_headers(&self) -> (HeaderMap, Sha256dHash) {
        let mut headers = HeaderMap::new();
        let mut blockhash = Sha256dHash::from_hex(
            "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
        ).unwrap(); // genesis block hash
        loop {
            let data = self.get(&format!("headers/2000/{}.bin", blockhash.be_hex_string()));
            assert!(!data.is_empty());
            let num_of_headers = data.len() / HEADER_SIZE;
            let mut decoder = RawDecoder::new(Cursor::new(data));
            for _ in 0..num_of_headers {
                let header: BlockHeader =
                    ConsensusDecodable::consensus_decode(&mut decoder).unwrap();
                blockhash = header.bitcoin_hash();
                headers.insert(blockhash, header);
            }
            if num_of_headers == 1 {
                break;
            }
        }
        (headers, blockhash)
    }

    fn add_missing_headers(&self, indexed_headers: HeaderList) -> (HeaderMap, Sha256dHash) {
        let mut header_map = indexed_headers.into_map();
        let data = self.get("chaininfo.json");
        let reply: Value = from_slice(&data).unwrap();
        let bestblockhash_hex = reply.get("bestblockhash").unwrap().as_str().unwrap();
        let bestblockhash = Sha256dHash::from_hex(bestblockhash_hex).unwrap();
        let mut blockhash = bestblockhash;
        while !header_map.contains_key(&blockhash) {
            let data = self.get(&format!("headers/1/{}.bin", blockhash.be_hex_string()));
            let header: BlockHeader = deserialize(&data).unwrap();
            header_map.insert(blockhash, header);
            blockhash = header.prev_blockhash;
        }
        (header_map, bestblockhash)
    }

    pub fn enumerate_headers(&self, indexed_headers: Option<HeaderList>) -> HeaderList {
        let (mut header_map, mut blockhash) = match indexed_headers {
            Some(headers) => self.add_missing_headers(headers),
            None => self.get_all_headers(),
        };
        let mut header_list = VecDeque::<BlockHeader>::new();

        let null_hash = Sha256dHash::default();
        while blockhash != null_hash {
            let header: BlockHeader = header_map.remove(&blockhash).unwrap();
            blockhash = header.prev_blockhash;
            header_list.push_front(header);
        }
        assert!(header_map.is_empty());
        HeaderList {
            headers: enumerate(header_list).collect(),
        }
    }
}
