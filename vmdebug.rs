use vm;
use std::io::{stdout,Write};

pub fn prstack(vm : &mut vm::Vmem){
	for i in &vm.st {
		match *i {
			vm::Obj::I(ref x) => print!("{} ",x),
			vm::Obj::S(ref x) => print!("[{}] ",x),
		}
	}
	println!("")
}

pub fn printprompt(vm : &mut vm::Vmem){
	match vm.st[..].last()  {
		Some(&vm::Obj::I(ref x)) => print!("{} > ", x),
		Some(&vm::Obj::S(ref x)) => print!("[{}] > ", x),
		None => print!("> ")
	}
	stdout().flush().unwrap_or(())
}
