use std;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Mutex;
use vm::*;

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

fn initusers() -> HashMap<String, User> {
	let mut hm = HashMap::new();
	hm.insert(String::from("root"), User { psw: String::from("\0"), gid: String::from("root") });
	hm
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

fn mkdircore(cursor: &mut HashMap<String, Fnode>, mut it: std::str::Split<char>){
	if let Some(mut name) = it.next() {
		if name.is_empty() {
			mkdircore(cursor, it)
		}else{
			let nchack = match cursor.get_mut(name){
				Some(&mut Fnode::Children(ref mut nc)) => return mkdircore(nc, it),
				Some(_) => return println!("{}", name),
				None => {
					let mut nc = HashMap::new();
					mkdircore(&mut nc, it);
					nc
				}
			};
			cursor.insert(String::from(name), Fnode::Children(nchack));
		}
	}
}

fn mkdir(vm: &mut Vmem){
	if let Some(Obj::S(dnameraw)) = vm.st.pop() {
		if let Fnode::Children(ref mut cursor) = GSES.lock().unwrap().froot {
			println!("{}", dnameraw);
			mkdircore(cursor, pathfix(&dnameraw).split('/'))
		}
	}
}/*
fn ldircore(cursor : &HashMap<String, Fnode>){
	for key in cursor.keys() {
		println!("{}", key);
		if let Some(Fnode::Children(ref c)) = cursor.get(key) {
			ldircore(&c)
		}
	}
}*/
fn ldir(vm : &mut Vmem){
	//if let Some(Obj::S(dnameraw)) = vm.st.pop() {
		/*if let Fnode::Children(ref cursor) = GSES.lock().unwrap().froot {
			ldircore(cursor)
		}*/
		println!("{:?}", GSES.lock().unwrap().froot)
	//}
}

/*fn handleuop(vm: &mut Vmem, op: &str){
}*/

pub fn sysify(vm: &mut Vmem){
	//vm.uop = Some(handleuop); // For $PATH
	vm.ffi.insert("cd", chdir);
	vm.ffi.insert("wd", wdir);
	vm.ffi.insert("md", mkdir);
	vm.ffi.insert("ls", ldir);
	//vm.ffi.insert("fread", fread);
	//vm.ffi.insert("fwrite", fwrite);
	//vm.ffi.insert("fexec", fexec);
}
