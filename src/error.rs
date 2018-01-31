use std::io;
use {ethabi, hex, reqwest};

error_chain! {
	links {
		Ethabi(ethabi::Error, ethabi::ErrorKind);
	}

	foreign_links {
		Io(io::Error);
		Hex(hex::FromHexError);
    Reqwest(reqwest::Error);
	}
}