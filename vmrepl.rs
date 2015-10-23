use std::env;
use std::io::{BufRead,stdin};
mod vm;
mod vmdebug;

fn main() {
	let mut vm = vm::newvm();
	vm::vmexec(&mut vm, vm::VMPRELUDE);
	for arg in env::args() {
		vm::vmexec(&mut vm, &arg[..])
	}
	vm.ffi.insert("prstack", vmdebug::prstack);
	vmdebug::printprompt(&mut vm);
	let stdinref = stdin();
	for lineres in stdinref.lock().lines() {
		if let Ok(line) = lineres {
			vm::vmexec(&mut vm, &line[..]);
			vmdebug::printprompt(&mut vm)
		}
	}
}
