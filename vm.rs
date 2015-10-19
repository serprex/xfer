use std::char;
use std::collections::HashMap;
use std::io::{Read,stdin};
use std::vec::*;

#[derive(Clone)]
pub enum Obj{
	S(String),
	I(i64),
}
pub struct Vmem{
	pub st : Vec<Obj>,
	pub ws : HashMap<String, String>,
	pub ffi : HashMap<&'static str, fn(&mut Vmem)>,
}

fn add(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(a + b))
	}
}
fn sub(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(a - b))
	}
}
fn mul(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(a * b))
	}
}
fn divmod(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		if b != 0{
			vm.st.push(Obj::I(a/b));
			vm.st.push(Obj::I(a%b))
		}
	}
}
fn pick(vm : &mut Vmem){
	if let Some(Obj::I(top)) = vm.st.pop() {
		if top == 0 { vm.st.pop(); }
		else {
			let len = vm.st.len();
			if len > 0 { vm.st.swap_remove(len-1); }
		}
	}
}
fn sform(vm : &mut Vmem){
	let slen = vm.st.len();
	if slen < 2 { return vm.st.clear() }
	if let (Obj::I(popxi), Obj::I(basei)) = (vm.st[slen-1].clone(), vm.st[slen-2].clone()) {
		if popxi < 0 || basei < 0 { return }
		let (popx, base) = (popxi as usize, basei as usize);
		if popx+2 > slen || base+2 > slen { return }
		let (spopx, sbase) : (usize, usize) = (slen-popx-2, slen-base-2);
		for i in 0 .. popx {
			if let Obj::I(offs) = vm.st[spopx+i] {
				if offs >= 0 {
					let offu = offs as usize;
					if offu <= spopx { vm.st[spopx + i] = vm.st[spopx - offu].clone() }
				}
			}
		}
		for i in 0 .. popx {
			vm.st[sbase+i] = vm.st[spopx+i].clone();
		}
		vm.st.truncate(sbase+popx);
	}
}
fn printobj(vm : &mut Vmem){
	match vm.st.pop() {
		Some(Obj::I(ai)) => print!("{}", ai),
		Some(Obj::S(_as)) => print!("{}", _as),
		_ => println!("Stack underflow")
	}
}
fn u32char(u : u32) -> char{
	char::from_u32(u).unwrap_or('\u{fffd}')
}
fn printchr(vm : &mut Vmem){
	if let Some(Obj::I(ai)) = vm.st.pop() {
		print!("{}", u32char(ai as u32))
	}
}
fn pushdepth(vm : &mut Vmem){
	let len = vm.st.len() as i64;
	vm.st.push(Obj::I(len));
}
fn defword(vm : &mut Vmem){
	if let (Some(Obj::S(_as)), Some(Obj::S(_bs))) = (vm.st.pop(), vm.st.pop()) {
		vm.ws.insert(_as, _bs);
	}
}
fn execstr(vm : &mut Vmem){
	if let Some(Obj::S(code)) = vm.st.pop() {
		vmexec(vm, &code[..])
	}
}
fn getchr(vm : &mut Vmem){
	if let Some(Ok(c)) = stdin().bytes().next() {
		vm.st.push(Obj::I(c as i64))
	}
}
fn execword(op : &str, vm : &mut Vmem){
	if let Ok(val) = op.parse::<i64>(){
		return vm.st.push(Obj::I(val))
	}
	let fc = if let Some(func) = vm.ffi.get(op)
		{ Some(func.clone()) } else { None };
	if let Some(fc) = fc { fc(vm) }
	let wc = if let Some(wf) = vm.ws.get(op)
		{ Some(wf.clone()) } else { None };
	if let Some(wc) = wc { vmexec(vm, &wc[..]) }
}

pub static VMPRELUDE : &'static str = "[ 0 $ ] [popx] : \
[ 1 popx ] [pop] : \
[ 1 1 $ ] [dupx] : \
[ 1 dupx ] [dup] : \
[ 2 dupx ] [over] : \
[ 1 2 4 2 $ ] [swap] : \
[ 1 3 2 6 3 $ ] [rsh3] : \
[ 2 1 3 6 3 $ ] [lsh3] : \
[ ? . ] [if] : \
[ [] rsh3 if ] [iff] : \
[ -1 * ] [neg] : \
[ print 10 prchr ] [prln] :";
fn xdigit(c : u32) -> u32 {
	if c >= ('0' as u32) && c <= ('9' as u32) { c-('0' as u32) }
	else if c >= ('a' as u32) && c<= ('z' as u32) { c-('a' as u32)+10 }
	else { 16 }
}
fn parsestring(s : &str) -> (String, bool){
	let mut ret = String::new();
	let mut lc = '\x00';
	let mut esc = false;
	let mut hex = 0;
	let mut uni = 0;
	for c in s.chars() {
		if !esc && c == '\\'{
			lc = '\x00';
			esc = true
		} else {
			if esc {
				let xd = xdigit(c as u32);
				uni = if uni == 0 {
					if c == 'u' { 7 } else if xd<16 { 1 } else { -2 }
				} else { uni-1 };
				if uni >= 0 {
					if xd<16 { hex = (hex<<4)|xd }
					else { uni = -1 }
				}
				if uni <= 0 {
					if uni > -2 { ret.push(u32char(hex)) }
					if uni < 0 { ret.push(c) }
					esc = false
				}
			} else {
				lc = c;
				ret.push(c)
			}
		}
	}
	if lc != ']' { ret.push(' ') }
	else if !ret.is_empty() {
		let rl1 = ret.len()-1;
		ret.truncate(rl1);
	}
	(ret, lc == ']')
}
pub fn newvm() -> Vmem {
	let mut builtins : HashMap<&'static str, fn(&mut Vmem)> = HashMap::new();
	builtins.insert("+", add);
	builtins.insert("-", sub);
	builtins.insert("*", mul);
	builtins.insert("%/", divmod);
	builtins.insert("$", sform);
	builtins.insert("?", pick);
	builtins.insert("getch", getchr);
	builtins.insert("print", printobj);
	builtins.insert("prchr", printchr);
	builtins.insert("depth", pushdepth);
	builtins.insert(".", execstr);
	builtins.insert(":", defword);
	Vmem { st : Vec::new(), ffi : builtins, ws : HashMap::new() }
}
pub fn vmexec(vm : &mut Vmem, code : &str){
	let mut ops = code.split(' ');
	while let Some(op) = ops.next() {
		match op {
			"@" => vm.st.push(Obj::S(String::from(code))),
			"ret" => return,
			_ if op.starts_with("[") => {
				let mut s = String::new();
				let mut pm = 1;
				let (chunk, end) = parsestring(op);
				s.push_str(&chunk[1..]);
				if !end {
					while let Some(op) = ops.next() {
						let (chunk, end) = parsestring(op);
						if chunk.starts_with("[") { pm += 1 }
						s.push_str(&chunk[..]);
						if end {
							if pm == 1 { break }
							else {
								pm -= 1;
								s.push_str("] ")
							}
						}
					}
				}
				vm.st.push(Obj::S(s));
			},
			_ => execword(op, vm)
		}
	}
}
