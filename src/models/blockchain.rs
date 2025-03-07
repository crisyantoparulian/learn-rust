use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use uuid::Uuid;
// use std::collections::HashMap;
use md5;
use std::sync::{Arc, Mutex};
// use warp::Filter;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: String,
    pub timestamp: i64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
}

impl Block {
    fn new(data: String, previous_hash: String) -> Self {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().timestamp();
        let hash = Block::calculate_hash(&id, &timestamp, &data, &previous_hash);
        Block {
            id,
            timestamp,
            data,
            previous_hash,
            hash,
        }
    }

    fn calculate_hash(id: &str, timestamp: &i64, data: &str, previous_hash: &str) -> String {
        let input = format!("{}{}{}{}", id, timestamp, data, previous_hash);
        format!("{:x}", md5::compute(input))
    }
}

#[derive(Clone, Serialize)]
pub struct Blockchain {
    chain: Arc<Mutex<Vec<Block>>>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = Block::new(String::from("Genesis Block"), String::from("0"));
        Blockchain { 
            chain: Arc::new(Mutex::new(vec![genesis_block])), 
        }
    }

    pub fn add_block(&self, data: String) {

        if let Ok(mut chain) = self.chain.lock(){
            if let Some(previous_block) = chain.last() {
                let new_block = Block::new(data, previous_block.hash.clone());
                chain.push(new_block);
            }  else {
                eprintln!("Previous block not found");
            }         
        }else{
            eprintln!("Failed to lock the blockchain mutex");
        }
        
    }

    pub fn get_chain(&self) -> Vec<Block>{
        match self.chain.lock() {
            Ok(chain) => chain.clone(),
            Err(err) => {
                eprintln!("Mutex poisoned: {}", err);
                vec![] // Return an empty chain instead of panicking
            }
        }
    }
}