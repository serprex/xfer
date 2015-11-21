#[macro_use]
extern crate lazy_static;
use std::io::{BufRead,stdin};
mod vm;
mod vf;
mod vl;
mod vmsys;
mod vmdebug;
mod fsinit;

fn main() {
	vmsys::initfs();
	let mut vm = Default::default();
	vf::vmexec(&mut vm, vf::VMPRELUDE);
	vmsys::sysify(&mut vm);
	vm.ffi.insert("prstack", vmdebug::prstack);
	vmdebug::prprompt(&mut vm);
	let stdinref = stdin();
	let mut line = String::new();
	while let Ok(_) = stdinref.read_line(&mut line) {
		vf::vmexec(&mut vm, &line[..]);
		vmdebug::prprompt(&mut vm);
		line.clear()
	}
}
