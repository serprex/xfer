use std::char;
use std::collections::HashMap;
use std::io::{Read,stdin};
use std::iter::Iterator;
use std::vec::*;

#[derive(Clone)]
pub enum Obj{
	S(String),
	I(i64),
	A(Vec<Obj>),
}
pub struct Vmem{
	pub st : Vec<Obj>,
	pub vars : HashMap<String, Obj>,
	pub ws : HashMap<String, String>,
	pub ffi : HashMap<&'static str, fn(&mut Vmem)>,
	pub uop : Option<fn(&mut Vmem, &str)>,
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
fn band(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(a & b))
	}
}
fn bor(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(a | b))
	}
}
fn bxor(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(a ^ b))
	}
}
fn pick(vm : &mut Vmem){
	if let Some(Obj::I(top)) = vm.st.pop() {
		if top == 0 { vm.st.pop(); }
		else {
			let len = vm.st.len();
			if len > 1 { vm.st.swap_remove(len-2); }
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
		Some(Obj::A(_)) => print!("[A]"),
		None => println!("Stack underflow")
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
fn nth(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::S(b))) = (vm.st.pop(), vm.st.pop()){
		if let Some(ch) = b.chars().nth(a as usize) {
			vm.st.push(Obj::I(ch as i64))
		}
	}
}
fn len(vm : &mut Vmem){
	if let Some(Obj::S(a)) = vm.st.pop(){
		vm.st.push(Obj::I(a.len() as i64))
	}
}
fn gt(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(if a > b {1} else {0}))
	}
}
fn setvar(vm : &mut Vmem){
	if let (Some(Obj::S(s)),Some(o)) = (vm.st.pop(),vm.st.pop()) {
		vm.vars.insert(s, o);
	}
}
fn getvar(vm : &mut Vmem){
	if let Some(Obj::S(s)) = vm.st.pop() {
		if let Some(o) = vm.vars.get(&s) {
			vm.st.push(o.clone());
		}
	}
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
fn getline(vm : &mut Vmem){
	let mut s = String::new();
	if let Ok(_) = stdin().read_line(&mut s) { vm.st.push(Obj::S(s)) }
}
fn execword(op : &str, vm : &mut Vmem){
	if let Ok(val) = op.parse::<i64>(){
		return vm.st.push(Obj::I(val))
	}
	let fc = if let Some(func) = vm.ffi.get(op)
		{ Some(func.clone()) } else { None };
	if let Some(fc) = fc {
		fc(vm)
	}else{
		let wc = if let Some(wf) = vm.ws.get(op)
			{ Some(wf.clone()) } else { None };
		if let Some(wc) = wc {
			vmexec(vm, &wc[..])
		}else{
			if let Some(uop) = vm.uop {
				uop(vm, op)
			}
		}
	}
}
pub static VMPRELUDE : &'static str = "[0 $]'popx : \
[1 popx]'pop : \
[1 1 $]'dupnth : \
[1 dupnth]'dup : \
[2 dupnth]'over : \
[2 1 2 2 $]'dup2 : \
[3 2 1 3 3 $]'dup3 : \
[1 2 4 2 $]'swap : \
[1 3 2 6 3 $]'rsh3 : \
[2 1 3 6 3 $]'lsh3 : \
[? .]'if : \
[' rsh3 if]'iff : \
[-1 *]'neg : \
[1 0 lsh3 ?]'not : \
[0 1 lsh3 ?]'boo : \
[dup2 > rsh3 - | boo]'>= : \
[> not]'<= : \
[>= not]'< : \
[dup2 - not]'== : \
[dup2 - boo]'!= : \
[print 10 prchr]'prln :";
fn xdigit(c : u32) -> u32 {
	if c >= ('0' as u32) && c <= ('9' as u32) { c-('0' as u32) }
	else if c >= ('a' as u32) && c<= ('z' as u32) { c-('a' as u32)+10 }
	else { 16 }
}
fn parsestring(s : &str) -> String{
	let mut ret = String::new();
	let mut esc = false;
	let mut hex = 0;
	let mut uni = 0;
	for c in s.chars() {
		if !esc && c == '\\'{
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
			} else { ret.push(c) }
		}
	}
	ret
}
pub fn newvm() -> Vmem {
	let mut b : HashMap<&'static str, fn(&mut Vmem)> = HashMap::new();
	b.insert("+", add);
	b.insert("-", sub);
	b.insert("*", mul);
	b.insert("&", band);
	b.insert("|", bor);
	b.insert("^", bxor);
	b.insert("%/", divmod);
	b.insert("$", sform);
	b.insert("?", pick);
	b.insert("getline", getline);
	b.insert("print", printobj);
	b.insert("prchr", printchr);
	b.insert("depth", pushdepth);
	b.insert("nth", nth);
	b.insert("len", len);
	b.insert(">", gt);
	b.insert("=", setvar);
	b.insert("\\", getvar);
	b.insert(".", execstr);
	b.insert(":", defword);
	Vmem { st: Vec::new(), ffi: b, vars: HashMap::new(), ws: HashMap::new(), uop: None }
}
fn bracketmatch<T: Iterator<Item=(usize,char)>>(mut chars: T) -> usize {
	let mut idx: usize = 0;
	let mut pm: u32 = 0;
	while let Some((oi, ch)) = chars.next() {
		idx = oi+1;
		if ch == '\\' { chars.next(); }
		else {
			if ch == '[' { pm += 1 }
			else if ch == ']' {
				if pm == 0 { break }
				pm -= 1;
			}
		}
	}
	idx
}
fn tokenize(code: &str, opi: usize) -> (usize,usize){
	let mut chars = code[opi..].chars().enumerate().skip_while(|&(_,ch)| ch.is_whitespace());
	let fch = chars.next();
	match fch {
		None => (0, 0),
		Some((oi,'[')) => (opi+oi, opi+bracketmatch(chars)),
		Some((oi,_)) =>
			(opi+oi, opi+match chars.take_while(|&(_,ch)| !ch.is_whitespace()).last() {
				None => oi+1,
				Some((i,_)) => i+1,
			})
	}
}
pub fn vmexec(vm : &mut Vmem, code : &str){
	let mut opinext = 0;
	loop {
		let (opi, opend) = tokenize(code, opinext);
		if opend == 0 { return }
		opinext = opend;
		let op = &code[opi..opend];
		match op {
			"<@>" => vm.st.push(Obj::S(String::from(code))),
			"@>" => vm.st.push(Obj::S(String::from(&code[opi..]))),
			"<@" => vm.st.push(Obj::S(String::from(&code[..opi]))),
			"ret" => return,
			_ if op.starts_with("'") => vm.st.push(Obj::S(String::from(&op[1..]))),
			_ if op.starts_with("[") => vm.st.push(Obj::S(parsestring(&op[1..op.len()-1]))),
			_ => execword(op, vm)
		}
	}
}
