#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;

extern crate serde;
extern crate serde_json;
extern crate ethabi;
extern crate rustc_hex as hex;
extern crate reqwest;

mod error;

use std::fs::File;
use std::collections::HashMap;
use hex::ToHex;
use ethabi::param_type::ParamType;
use ethabi::token::{Token, Tokenizer, StrictTokenizer, LenientTokenizer};
use ethabi::{Contract, Function};
use error::Error;

fn main() {
    let values: &Vec<String> = &vec![String::from("yolo")];
    let encoded = encode_input("./src/abi.json", "set", values, false).unwrap();
    println!("{}", encoded);

    // Example of how to make JSON-RPC requests to POA network
    // TODO: Proper error handling
    // TODO: Call the correct JSON-RPC method
    // TODO: Connection pooling
    let mut map = HashMap::new();
    map.insert("jsonrpc", "2.0");
    map.insert("method", "web3_clientVersion");
    map.insert("params", "[]");
    map.insert("id", "67");

    // TODO: Don't define inline
    #[derive(Deserialize, Debug)]
    struct TestJSONRPCResponse {
        jsonrpc: String,
        id: u32,
    }

    let client = reqwest::Client::new();
    let response: TestJSONRPCResponse = client.post("https://sokol.poa.network")
        .json(&map)
        .send()
        .unwrap()
        .json()
        .unwrap();
    println!("body = {:?}", response);
}

// from: https://github.com/paritytech/ethabi/blob/master/cli/src/main.rs
// encode_input accepts a path to a JSON ABI file, the name of the function for
// which we'd like to encode input, and the values of the arguments we want to encode
// for the specified function (in the order that the parameters of the function
// are defined)
fn encode_input(path: &str, function: &str, values: &[String], lenient: bool) -> Result<String, Error> {
    let function = load_function(path, function)?;

    // Zip the functions parameters together with their values (arguments in the form of &str)
    let params: Vec<_> = function.inputs.iter()
        .map(|param| param.kind.clone())
        .zip(values.iter().map(|v| v as &str))
        .collect();

    let tokens = parse_tokens(&params, lenient);
    let result = function.encode_input(&tokens)?;
    Ok(result.to_hex())
}

// from: https://github.com/paritytech/ethabi/blob/master/cli/src/main.rs
// TODO: Figure out how to return a result instead of force unwrapping
// load_function accepts a path to a JSON ABI file and the name of the function
// to load, and returns a Function which can be used to encode the desired input.
fn load_function(path: &str, function: &str) -> Result<Function, Error> {
    // ? is syntactic sugar which will early return with an error if the operation
    // fails.
    let file = File::open(path).unwrap();
    let contract = Contract::load(file)?;
    let function = contract.function(function)?.clone();
    Ok(function)
}

// from: https://github.com/paritytech/ethabi/blob/master/cli/src/main.rs
// parse_tokens accepts an array of tuples of (ParamType, &str) I.E the type
// of the parameter we want to encode as well as its string representation
// and it returns a vector of Token that we can pass to the encode_input
// method of the Function type to encode our function call.
fn parse_tokens(params: &[(ParamType, &str)], lenient: bool) -> Vec<Token> {
    params.iter()
        .map(|&(ref param, value)| match lenient {
            // TODO: Don't unwrap here
            true => LenientTokenizer::tokenize(param, value).unwrap(),
            false => StrictTokenizer::tokenize(param, value).unwrap()
        })
        .collect::<_>()
}