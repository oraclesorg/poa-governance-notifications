extern crate ethabi;
extern crate rustc_hex as hex;
extern crate reqwest;

use std::fs::File;
use hex::ToHex;
use ethabi::param_type::ParamType;
use ethabi::token::{Token, Tokenizer, StrictTokenizer, LenientTokenizer};
use ethabi::{Contract, Function};

fn main() {
    let values: &Vec<String> = &vec![String::from("yolo")];
    println!("{}", encode_input("./src/abi.json", "set", values, false));

    let mut resp = reqwest::get("https://www.rust-lang.org").unwrap();
    assert!(resp.status().is_success());
    let body = resp.text().unwrap();
    println!("body = {:?}", body);
}

// from: https://github.com/paritytech/ethabi/blob/master/cli/src/main.rs
// encode_input accepts a path to a JSON ABI file, the name of the function for
// which we'd like to encode input, and the values of the arguments we want to encode
// for the specified function (in the order that the parameters of the function
// are defined)
fn encode_input(path: &str, function: &str, values: &[String], lenient: bool) -> String {
    let function = load_function(path, function);

    // Zip the functions parameters together with their values (arguments in the form of &str)
    let params: Vec<_> = function.inputs.iter()
        .map(|param| param.kind.clone())
        .zip(values.iter().map(|v| v as &str))
        .collect();

    let tokens = parse_tokens(&params, lenient);
    let result = function.encode_input(&tokens).unwrap();
    result.to_hex()
}

// from: https://github.com/paritytech/ethabi/blob/master/cli/src/main.rs
// TODO: Figure out how to return a result instead of force unwrapping
// load_function accepts a path to a JSON ABI file and the name of the function
// to load, and returns a Function which can be used to encode the desired input.
fn load_function(path: &str, function: &str) -> Function {
    // ? is syntactic sugar which will early return with an error if the operation
    // fails.
    let file = File::open(path).unwrap();
    let contract = Contract::load(file).unwrap();
    contract.function(function).unwrap().clone()
}

// from: https://github.com/paritytech/ethabi/blob/master/cli/src/main.rs
// parse_tokens accepts an array of tuples of (ParamType, &str) I.E the type
// of the parameter we want to encode as well as its string representation
// and it returns a vector of Token that we can pass to the encode_input
// method of the Function type to encode our function call.
fn parse_tokens(params: &[(ParamType, &str)], lenient: bool) -> Vec<Token> {
    params.iter()
        .map(|&(ref param, value)| match lenient {
            true => LenientTokenizer::tokenize(param, value).unwrap(),
            false => StrictTokenizer::tokenize(param, value).unwrap()
        })
        .collect::<_>()
}