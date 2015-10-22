use std::*;
use std::collections::HashMap;
use vm::*;

enum Fdata {
	Children(Vec<Fnode>),
	Text(String),
	Bytes(Vec<u8>),
}
struct Fnode{
	path : String,
	parent : Option<Box<Fnode>>,
	data : Fdata,
}
#[derive(Default)]
struct Session{
	uid : u16,
	gid : u16,
	dir : String,
	usr : String,
	psw : String,
}

fn parsesession(ses : &String) -> Session {
	let ses = ses.split('\n');
	let mut ret : Session = Default::default();
	if let Some(uid) = ses.next() { ret.uid = uid.parse::<u16>().unwrap_or(0) }
	if let Some(gid) = ses.next() { ret.gid = gid.parse::<u16>().unwrap_or(0) }
	if let Some(dir) = ses.next() { ret.dir = String::from(dir) }
	if let Some(usr) = ses.next() { ret.usr = String::from(usr) }
	if let Some(psw) = ses.next() { ret.psw = String::from(psw) }
	ret
}

fn stringsession(ses : &Session) -> String {
	format!("{}\n{}\n{}\n{}\n{}", ses.uid, ses.gid, ses.dir, ses.usr, ses.psw)
}

fn loadses(vm : &mut Vmem) -> Session {
	if let Some(sesbox) = vm.ws.get(&String::from("SESSION")){
		parsesession(sesbox)
	}else{
		let session = Session{uid:0, gid:0, dir:String::from("/"), usr:String::from(""), psw:String::from("")};
		vm.ws.insert(String::from("SESSION"), stringsession(&session));
		session
	}
}

fn pathfixtrim(part : &str) -> String{
	String::from(if let Some(idx) = part.rfind('/')
		{ &part[..idx] } else { part })
}

fn pathfix(path : &String) -> String{
	path.split("/../").map(pathfixtrim).collect::<Vec<_>>().join("/")
}

fn chdir(vm : &mut Vmem){
	let arg = if let Some(Obj::S(sarg)) = vm.st.pop()
		{ sarg.clone() } else { return };
	let mut ses = loadses(vm);
	if arg.starts_with("/") {
		ses.dir = arg
	}else{
		ses.dir.push_str(&arg[..]);
		if !ses.dir.ends_with("/") { ses.dir.push('/') }
	}
	ses.dir = pathfix(&ses.dir);
}

fn wdir(vm : &mut Vmem){
	let ses = loadses(vm);
	vm.st.push(Obj::S(ses.dir.clone()))
}

/*fn handleuop(vm : &mut Vmem, op : &str){
	
}*/

pub fn sysify(vm : &mut Vmem){
	//vm.uop = Some(handleuop);
	vm.ffi.insert("cd", chdir);
	vm.ffi.insert("wd", wdir);
	//vm.ffi.insert("fread", fread);
	//vm.ffi.insert("fwrite", fwrite);
	//vm.ffi.insert("fexec", fexec);
}
