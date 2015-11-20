use std;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Mutex;
use vm::*;
use vf;

#[derive(Debug)]
enum Fnode{
	No,
	Children(HashMap<String,Fnode>),
	Text(String),
	Bytes(Vec<u8>),
}
#[derive(Debug)]
struct Group{
	name: String,
	children: Vec<Group>,
}
#[derive(Debug)]
struct User{
	psw: String,
	gid: String,
}
#[derive(Debug)]
struct Session{
	dir: String,
	user: String,
	froot: Fnode,
	groot: Group,
	users: HashMap<String, User>,
}

lazy_static! {
	static ref GSES: Mutex<Session> = Mutex::new(Session{
		dir: String::from("/"),
		user: String::from("root"),
		froot: Fnode::Children(HashMap::new()),
		groot: Group { name: String::from("root"), children: Vec::new() },
		users: HashMap::new(),
	});
}

pub fn initfs() {
	let mut fsrc = String::new();
	if let Ok(ref mut f) = std::fs::File::open("fs")
		{ f.read_to_string(&mut fsrc); }
	else if let Ok(ref mut f) = std::fs::File::open("fsinit")
		{ f.read_to_string(&mut fsrc); }
	else { std::process::exit(0) }
	let mut vm = Default::default();
	vf::vmexec(&mut vm, vf::VMPRELUDE);
	sysify(&mut vm);
	vf::vmexec(&mut vm, &fsrc[..])
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
		{ ses.dir.push_str(&arg) }
	ses.dir = pathfix(&ses.dir);
}

fn wdir(vm: &mut Vmem){
	vm.st.push(Obj::S(GSES.lock().unwrap().dir.clone()))
}

fn mkdircore(cursor: &mut HashMap<String, Fnode>, mut it: std::str::Split<char>){
	if let Some(name) = it.next() {
		if name.is_empty() {
			mkdircore(cursor, it)
		}else{
			let ncentry = cursor.entry(String::from(name)).or_insert_with(|| Fnode::Children(HashMap::new()));
			if let Fnode::Children(ref mut nc) = *ncentry
				{ mkdircore(nc, it) } else { println!("{}", name) }
		}
	}
}

fn mkdir(vm: &mut Vmem){
	if let Some(Obj::S(dnameraw)) = vm.st.pop() {
		if let Fnode::Children(ref mut cursor) = GSES.lock().unwrap().froot {
			mkdircore(cursor, pathfix(&dnameraw).split('/'))
		}
	}
}
fn ldircore(cursor: &mut HashMap<String, Fnode>, mut it: std::str::Split<char>, vm: &mut Vmem){
	if let Some(name) = it.next() {
		if name.is_empty() {
			ldircore(cursor, it, vm)
		}else{
			let ncentry = cursor.entry(String::from(name)).or_insert_with(|| Fnode::Children(HashMap::new()));
			if let Fnode::Children(ref mut nc) = *ncentry
				{ ldircore(nc, it, vm) } else { vm.st.push(Obj::E) }
		}
	} else { vm.st.push(Obj::A(cursor.keys().map(|x| Obj::S(x.clone())).collect())) }
}
fn ldir(vm: &mut Vmem){
	if let Some(Obj::S(dnameraw)) = vm.st.pop() {
		let dname = pathfix(&dnameraw);
		if let Some(ridx) = dname[..dname.len()-1].rfind('/') {
			if let Fnode::Children(ref mut cursor) = GSES.lock().unwrap().froot {
				ldircore(cursor, String::from(&dname[..ridx]).split('/'), vm)
			}
		}
	}
}
fn freadcore(cursor: &mut HashMap<String, Fnode>, mut it: std::str::Split<char>, fname: &str, vm: &mut Vmem){
	if let Some(name) = it.next() {
		if name.is_empty() {
			freadcore(cursor, it, fname, vm)
		}else{
			let ncentry = cursor.entry(String::from(name)).or_insert_with(|| Fnode::Children(HashMap::new()));
			if let Fnode::Children(ref mut nc) = *ncentry
				{ freadcore(nc, it, fname, vm) } else { vm.st.push(Obj::E) }
		}
	}else if let Some(&Fnode::Text(ref content)) = cursor.get(fname) {
		vm.st.push(Obj::S(content.clone()))
	}
}
fn fread(vm: &mut Vmem){
	if let Some(Obj::S(dnameraw)) = vm.st.pop() {
		let dname = pathfix(&dnameraw);
		if let Some(ridx) = dname[..dname.len()-1].rfind('/') {
			if let Fnode::Children(ref mut cursor) = GSES.lock().unwrap().froot {
				freadcore(cursor, String::from(&dname[..ridx]).split('/'), &dname[ridx+1..dname.len()-1], vm)
			}
		}
	}
}
fn fwritecore(cursor: &mut HashMap<String, Fnode>, mut it: std::str::Split<char>, fname: &str, content: String){
	if let Some(name) = it.next() {
		if name.is_empty() {
			fwritecore(cursor, it, fname, content)
		}else{
			let ncentry = cursor.entry(String::from(name)).or_insert_with(|| Fnode::Children(HashMap::new()));
			if let Fnode::Children(ref mut nc) = *ncentry
				{ fwritecore(nc, it, fname, content) } else { println!("{}", name) }
		}
	}else{ cursor.insert(String::from(fname), Fnode::Text(content)); }
}
fn fwrite(vm: &mut Vmem){
	if let (Some(Obj::S(dnameraw)), Some(Obj::S(content))) = (vm.st.pop(),vm.st.pop()) {
		let dname = pathfix(&dnameraw);
		if let Some(ridx) = dname[..dname.len()-1].rfind('/') {
			if let Fnode::Children(ref mut cursor) = GSES.lock().unwrap().froot {
				mkdircore(cursor, String::from(&dname[..ridx]).split('/'));
				fwritecore(cursor, String::from(&dname[..ridx]).split('/'), &dname[ridx+1..dname.len()-1], content)
			}
		}
	}
}
fn rmcore(cursor: &mut HashMap<String, Fnode>, mut it: std::str::Split<char>, fname: &str){
	if let Some(name) = it.next() {
		if name.is_empty() {
			rmcore(cursor, it, fname)
		}else{
			let ncentry = cursor.entry(String::from(name)).or_insert_with(|| Fnode::Children(HashMap::new()));
			if let Fnode::Children(ref mut nc) = *ncentry
				{ rmcore(nc, it, fname) }
		}
	} else { cursor.remove(&String::from(fname)); }
}
fn rm(vm: &mut Vmem){
	if let Some(Obj::S(dnameraw)) = vm.st.pop() {
		let dname = pathfix(&dnameraw);
		if let Some(ridx) = dname[..dname.len()-1].rfind('/') {
			if let Fnode::Children(ref mut cursor) = GSES.lock().unwrap().froot {
				rmcore(cursor, String::from(&dname[..ridx]).split('/'), &dname[ridx+1..dname.len()-1])
			}
		}
	}
}
fn mkuser(vm: &mut Vmem){
	if let (Some(Obj::S(uname)), Some(Obj::S(upsw)), Some(Obj::S(ugid))) = (vm.st.pop(), vm.st.pop(), vm.st.pop()) {
		let mut ses = GSES.lock().unwrap();
		ses.users.entry(uname.clone()).or_insert(User{psw: upsw, gid: ugid});
	}
}
fn deluser(vm: &mut Vmem){
	if let Some(Obj::S(uname)) = vm.st.pop() {
		let mut ses = GSES.lock().unwrap();
		ses.users.remove(&uname);
	}
}
fn login(vm: &mut Vmem){
	if let (Some(Obj::S(uname)), Some(Obj::S(upsw))) = (vm.st.pop(), vm.st.pop()) {
		let mut ses = GSES.lock().unwrap();
		let n = if let Some(ref u) = ses.users.get(&uname) {
			u.psw == upsw
		} else { false };
		if n { ses.user = uname }
	}
}

fn handleuop(vm: &mut Vmem, op: &str){
	let mut bpath = String::from("bin/");
	bpath.push_str(op);
	vm.st.push(Obj::S(bpath));
	vf::vmexec(vm, "fread .")
}

pub fn sysify(vm: &mut Vmem){
	vm.uop = Some(handleuop);
	vm.ffi.insert("cd", chdir);
	vm.ffi.insert("wd", wdir);
	vm.ffi.insert("ls", ldir);
	vm.ffi.insert("mkdir", mkdir);
	vm.ffi.insert("fread", fread);
	vm.ffi.insert("fwrite", fwrite);
	vm.ffi.insert("rm", rm);
	vm.ffi.insert("mkuser", mkuser);
	vm.ffi.insert("deluser", deluser);
	vm.ffi.insert("login", login);
}
