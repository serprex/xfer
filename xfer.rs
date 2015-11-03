#[macro_use]
extern crate lazy_static;
use std::io::{BufRead,stdin};
mod vm;
mod vmsys;
mod vmdebug;

fn main() {
	vmsys::initfs();
	let mut vm = vm::newvm();
	vm::vmexec(&mut vm, vm::VMPRELUDE);
	vmsys::sysify(&mut vm);
	vm.ffi.insert("prstack", vmdebug::prstack);
	vmdebug::prprompt(&mut vm);
	let stdinref = stdin();
	for lineres in stdinref.lock().lines() {
		if let Ok(line) = lineres {
			vm::vmexec(&mut vm, &line[..]);
			vmdebug::prprompt(&mut vm)
		}
	}
}
