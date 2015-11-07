use vm;
use std::io::{stdout,Write};

pub fn objrepr(o: &vm::Obj) -> String {
	match o {
		&vm::Obj::E => String::from("E"),
		&vm::Obj::I(ref x) => format!("{}", x),
		&vm::Obj::S(ref x) => format!("[{}]", x),
		&vm::Obj::A(ref x) => {
			let mut s = String::new();
			for a in x {
				s.push_str(&objrepr(a));
				s.push(' ')
			}
			s.pop();
			format!("({})", s)
		}
	}
}

pub fn prstack(vm: &mut vm::Vmem){
	for o in &vm.st {
		print!("{} ", objrepr(o));
	}
	println!("")
}

pub fn prprompt(vm: &mut vm::Vmem){
	match vm.st[..].last()  {
		Some(o) => print!("{} > ", objrepr(o)),
		None => print!("> ")
	}
	stdout().flush().unwrap_or(())
}
