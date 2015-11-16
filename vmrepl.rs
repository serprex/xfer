use std::collections::{HashMap};
use std::io::{BufRead,stdin};
mod vm;
mod vf;
mod vl;
mod vmdebug;

fn pruop(_ : &mut vm::Vmem, op: &str){
	println!("op? {}", op);
}
fn exit(_: &mut vm::Vmem){
	std::process::exit(0)
}
fn main() {
	let mut vm: vm::Vmem = Default::default();
	let mut vlprefixes = HashMap::new();
	vm.vars.push(HashMap::new());
	vf::forthify(&mut vm.ffi);
	vf::vmexec(&mut vm, vf::VMPRELUDE);
	vl::lispify(&mut vm.ffi);
	vl::vmexec(&mut vm, &mut vlprefixes, vl::VMPRELUDE);
	vm.uop = Some(pruop);
	vm.ffi.insert("prstack", vmdebug::prstack);
	vm.ffi.insert("exit", exit);
	vmdebug::prprompt(&mut vm);
	let stdinref = stdin();
	let mut line = String::new();
	while let Ok(_) = stdinref.read_line(&mut line) {
		let islisp = line.starts_with("(");
		if islisp { vl::vmexec(&mut vm, &mut vlprefixes, &line[..]) }
		else { vf::vmexec(&mut vm, &line[..]) }
		vmdebug::prprompt(&mut vm);
		line.clear()
	}
}
