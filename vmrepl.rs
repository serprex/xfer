use std::env;
mod vm;

fn main() {
	match env::args().nth(1) {
		Some(code) => vm::vmstart(&code[..]),
		None => panic!("arg1 must be code")
	}
}
