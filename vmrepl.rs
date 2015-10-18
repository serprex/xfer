use std::env;
use std::io::{BufRead,Write,stdin,stdout};
mod vm;

fn main() {
	match env::args().nth(1) {
		Some(code) => vm::vmstart(&code[..]),
		None => {
			let mut vm = vm::newvm();
			vm::vmexec(&mut vm, vm::VMPRELUDE);
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
					match vm.st[..].last()  {
						Some(&vm::Obj::I(ref x)) => print!("{}", x),
						Some(&vm::Obj::S(ref x)) => print!("[{}]", x),
						None => ()
					}
					print!(" > ");
					stdout().flush().unwrap();
				}
			}
			/*let mut line = String::new();
			while let Ok(_) = stdin().read_line(&mut line) {
				print!("{} 123",line);
				vm::vmexec(&mut vm, &line[..])
			}*/
		}
	}
}
