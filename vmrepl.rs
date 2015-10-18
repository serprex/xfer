use std::env;
use std::io::{BufRead,Write,stdin,stdout};
mod vm;

fn printprompt(vm : &mut vm::Vmem){
	match vm.st[..].last()  {
		Some(&vm::Obj::I(ref x)) => print!("{}", x),
			Some(&vm::Obj::S(ref x)) => print!("[{}]", x),
			None => ()
	}
	print!(" > ");
	stdout().flush().unwrap()
}

fn main() {
	let mut vm = vm::newvm();
	vm::vmexec(&mut vm, vm::VMPRELUDE);
	for arg in env::args() {
		vm::vmexec(&mut vm, &arg[..])
	}
	printprompt(&mut vm);
	let stdinref = stdin();
	for lineres in stdinref.lock().lines() {
		if let Ok(line) = lineres {
			if line == "#stack"{
				for i in &vm.st {
					match *i {
						vm::Obj::I(ref x) => print!("{} ",x),
						vm::Obj::S(ref x) => print!("[{}] ",x),
					}
				}
				println!("{}","");
			}else{
				vm::vmexec(&mut vm, &line[..])
			}
			printprompt(&mut vm);
		}
	}
}
