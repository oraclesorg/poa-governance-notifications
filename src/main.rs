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
use hex::{FromHex, ToHex};
use ethabi::param_type::ParamType;
use ethabi::token::{Token, Tokenizer, StrictTokenizer, LenientTokenizer};
use ethabi::{Contract, Function};
use error::Error;

// Structure of JSON RPC Request
#[derive(Serialize, Debug)]
struct JSONRPCRequest {
    jsonrpc: String,
    method: String,
    params: (EthCallParams, String),
    id: u32,
}

// Structure of a JSON RPC Response
#[derive(Deserialize, Debug)]
struct JSONRPCResponse {
    jsonrpc: String,
    id: u32,
    result: String,
}

// Structure of the parameters for a JSON RPC Request of type eth_Call
#[derive(Serialize, Debug)]
struct EthCallParams {
    to: String,
    data: String,
}

fn main() {
    let values: &Vec<String> = &vec![];
    // TODO: Config
    let abi_file = "./src/voting_to_change_keys.abi.json";
    let function_name = "getBallotLimitPerValidator";
    let encoded = encode_input(abi_file, function_name, values, false).unwrap();
    println!("{}", encoded);

    // TODO: Config
    let response = make_eth_call(String::from("0x49df4ec19243263e5db22da5865b4f482b8323a0"), String::from("0x") + &encoded).unwrap();
    println!("{:?}", response);

    // TODO: More efficient way to do this
    let mut result = response.result.clone();
    result.remove(0);
    result.remove(0);
    let encoded_result = result.from_hex().unwrap();
    let decoded_output = decode_output(abi_file, function_name, &encoded_result).unwrap();
    println!("{:?}", decoded_output);
}

// TODO: Connection pooling / client re-use
fn make_eth_call(address: String, data: String) -> Result<JSONRPCResponse, Error>{
    let client = reqwest::Client::new();
    let request = JSONRPCRequest{
        jsonrpc: String::from("2.0"),
        method: String::from("eth_call"),
        params: (EthCallParams{
            to: address,
            data: data,
        }, String::from("latest")),
        id: 1,
    };

    // TODO: Config
    let response: JSONRPCResponse = client.post("https://sokol.poa.network")
        .json(&request)
        .send()?
        .json()?;
    
    Ok(response)
}

// from: https://github.com/paritytech/ethabi/blob/master/cli/src/main.rs
// encode_input accepts a path to a JSON ABI file, the name of the function for
// which we'd like to encode input, and the values of the arguments we want to encode
// for the specified function (in the order that the parameters of the function
// are defined)
fn encode_input(path: &str, function: &str, values: &[String], lenient: bool) -> Result<String, Error> {
    let function = load_function(path, function)?;
    println!(0);

    // Zip the functions parameters together with their values (arguments in the form of &str)
    let params: Vec<_> = function.inputs.iter()
        .map(|param| param.kind.clone())
        .zip(values.iter().map(|v| v as &str))
        .collect();

    println!(1);
    let tokens = parse_tokens(&params, lenient);
    println!(2);
    let result = function.encode_input(&tokens)?;
    println!(3);
    Ok(result.to_hex())
}

fn decode_output(path: &str, function: &str, data: &[u8]) -> Result<Vec<Token>, Error> {
    let function = load_function(path, function)?;
    let result = function.decode_output(data)?;
    Ok(result)
}

// from: https://github.com/paritytech/ethabi/blob/master/cli/src/main.rs
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

// TODO: Return result instead of force unwrapping
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