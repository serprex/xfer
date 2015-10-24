use std::collections::HashMap;
use std::sync::Mutex;
use vm::*;

enum Fdata {
	Children(Vec<Fnode>),
	Text(String),
	Bytes(Vec<u8>),
}
struct Fnode{
	path: String,
	parent: Option<Box<Fnode>>,
	data: Fdata,
}
#[derive(Default)]
struct Session{
	uid: u16,
	gid: u16,
	dir: String,
	usr: String,
	psw: String,
}

lazy_static! {
	static ref GSES: Mutex<Session> = Mutex::new(Session{uid:0, gid:0, dir:String::from("/"), usr:String::new(), psw:String::new()});
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
