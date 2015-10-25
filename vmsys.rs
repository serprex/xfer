use std;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Mutex;
use vm::*;

enum Fdata {
	No,
	Children(Vec<Fnode>),
	Text(String),
	Bytes(Vec<u8>),
}
struct Fnode{
	path: String,
	parent: Option<Box<Fnode>>,
	data: Fdata,
}
struct Group{
	name: String,
	children: Vec<Group>,
}
struct Session{
	gid: Group,
	dir: String,
	usr: String,
	psw: String,
	root: Fnode,
}

lazy_static! {
	static ref GSES: Mutex<Session> = Mutex::new(Session{
		gid: Group{ name: String::from("root"), children: Vec::new() },
		dir: String::from("/"),
		usr: String::new(),
		psw: String::new(),
		root: Fnode { path: String::new(), parent: None, data: Fdata::No },
	});
}

pub fn initfs() {
	let mut fsrc = String::new();
	if let Ok(ref mut f) = std::fs::File::open("fs")
		{ f.read_to_string(&mut fsrc); }
	else if let Ok(ref mut f) = std::fs::File::open("fsinit")
		{ f.read_to_string(&mut fsrc); }
	else { std::process::exit(0) }
	let mut vm = newvm();
	vmexec(&mut vm, VMPRELUDE);
	sysify(&mut vm);
	vmexec(&mut vm, &fsrc[..])
}

fn pathfix(path: &String) -> String{
	let mut ret = String::new();
	for part in path.split("/"){
		if part == ".." {
			if let Some(ridx) = ret.rfind('/'){ ret.truncate(ridx) }
		} else if !part.is_empty() && part != "." {
			ret.push('/');
			ret.push_str(part)
		}
	}
	ret.push('/');
	ret
}

fn chdir(vm: &mut Vmem){
	let arg = if let Some(Obj::S(sarg)) = vm.st.pop()
		{ sarg.clone() } else { return };
	let ref mut ses = *GSES.lock().unwrap();
	if arg.starts_with("/")
		{ ses.dir = arg } else
		{ ses.dir.push_str(&arg[..]) }
	ses.dir = pathfix(&ses.dir);
}

fn wdir(vm: &mut Vmem){
	vm.st.push(Obj::S(GSES.lock().unwrap().dir.clone()))
}

/*fn handleuop(vm: &mut Vmem, op: &str){
}*/

pub fn sysify(vm: &mut Vmem){
	//vm.uop = Some(handleuop); // For $PATH
	vm.ffi.insert("cd", chdir);
	vm.ffi.insert("wd", wdir);
	//vm.ffi.insert("fread", fread);
	//vm.ffi.insert("fwrite", fwrite);
	//vm.ffi.insert("fexec", fexec);
}
