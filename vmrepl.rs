use std::env;
use std::io::{BufRead,stdin};
mod vm;
mod vmdebug;

fn pruop(_ : &mut vm::Vmem, op: &str){
	println!("op? {}", op);
}
fn exit(_: &mut vm::Vmem){
	std::process::exit(0)
}
fn main() {
	let mut vm = vm::newvm();
	vm::vmexec(&mut vm, vm::VMPRELUDE);
	for arg in env::args() {
		vm::vmexec(&mut vm, &arg[..])
	}
	vm.uop = Some(pruop);
	vm.ffi.insert("prstack", vmdebug::prstack);
	vm.ffi.insert("exit", exit);
	vmdebug::prprompt(&mut vm);
	let stdinref = stdin();
	let mut line = String::new();
	while let Ok(_) = stdinref.read_line(&mut line) {
		vm::vmexec(&mut vm, &line[..]);
		vmdebug::prprompt(&mut vm);
		line.clear()
	}
}
