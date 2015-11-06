use std::char;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{Read,stdin};
use std::iter::Iterator;
use std::vec::*;

#[derive(Clone)]
pub enum Obj{
	E,
	I(i64),
	S(String),
	A(Vec<Obj>),
}
pub struct Vmem{
	pub st : Vec<Obj>,
	pub vars : Vec<HashMap<String, Obj>>,
	pub ws : HashMap<String, String>,
	pub ffi : HashMap<&'static str, fn(&mut Vmem)>,
	pub uop : Option<fn(&mut Vmem, &str)>,
}

fn add(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(a + b))
	} else { vm.st.push(Obj::E) }
}
fn sub(vm : &mut Vmem){
	if let (Some(ao),Some(bo)) = (vm.st.pop(), vm.st.pop()){
		match (ao,bo) {
			(Obj::I(a),Obj::I(b)) => vm.st.push(Obj::I(a - b)),
			(Obj::S(a),Obj::S(b)) => vm.st.push(Obj::I(match a.cmp(&b){
				Ordering::Less => -1,
				Ordering::Equal => 0,
				Ordering::Greater => 1,
			})),
			_ => vm.st.push(Obj::E)
		}
	}
}
fn mul(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(a * b))
	} else { vm.st.push(Obj::E) }
}
fn divmod(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		if b != 0{
			vm.st.push(Obj::I(a/b));
			vm.st.push(Obj::I(a%b))
		} else { vm.st.push(Obj::E) }
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
		Some(Obj::A(_)) => print!("A"),
		Some(Obj::E) => print!("E"),
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
fn mka(vm : &mut Vmem){
	vm.st.push(Obj::A(Vec::new()))
}
fn nth(vm : &mut Vmem){
	if let Some(Obj::I(n)) = vm.st.pop(){
		let r = match vm.st.last() {
			Some(&Obj::S(ref s)) => Obj::I(s.chars().nth(n as usize).unwrap_or('\0') as i64),
			Some(&Obj::A(ref a)) => a.get(n as usize).unwrap_or(&Obj::I(0)).clone(),
			_ => return
		};
		vm.st.push(r)
	}
}
fn siphon(vm : &mut Vmem) {
	let n = if let Some(Obj::I(n)) = vm.st.pop() { n } else { 0 };
	for _ in 0..n {
		let len = vm.st.len();
		vm.st.swap(len-2, len-1);
		pusha(vm)
	}
}
fn pusha(vm : &mut Vmem){
	if let Some(o) = vm.st.pop() {
		if let Some(&mut Obj::A(ref mut a)) = vm.st.last_mut() {
			a.push(o)
		}
	}
}
fn popa(vm : &mut Vmem){
	let ap = if let Some(&mut Obj::A(ref mut a)) = vm.st.last_mut()
		{ a.pop() } else { return };
	if let Some(apo) = ap { vm.st.push(apo) }
}
fn nthset(vm : &mut Vmem){
	if let Some(Obj::I(idx)) = vm.st.pop() {
		if let Some(o) = vm.st.pop() {
			if let Some(&mut Obj::A(ref mut a)) = vm.st.last_mut() {
				a[idx as usize] = o
			}
		}
	}
}
fn len(vm : &mut Vmem){
	if let Some(o) = vm.st.pop(){
		vm.st.push(Obj::I(match o {
			Obj::S(s) => s.len() as i64,
			Obj::A(a) => a.len() as i64,
			_ => -1
		}));
	}
}
fn gt(vm : &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(if a > b {1} else {0}))
	}
}
fn setvar(vm : &mut Vmem){
	if let (Some(Obj::S(s)),Some(o)) = (vm.st.pop(),vm.st.pop()) {
		if let Some(mut var) = vm.vars.last_mut() {
			var.insert(s, o);
		}
	}
}
fn getvar(vm : &mut Vmem){
	if let Some(Obj::S(s)) = vm.st.pop() {
		for vars in vm.vars.iter().rev() {
			if let Some(o) = vars.get(&s) {
				return vm.st.push(o.clone())
			}
		}
	}
}
fn gettype(vm : &mut Vmem){
	let t = Obj::I(match vm.st.pop() {
		Some(Obj::E) => 0,
		Some(Obj::I(_)) => 1,
		Some(Obj::S(_)) => 2,
		Some(Obj::A(_)) => 3,
		None => -1
	});
	vm.st.push(t)
}
fn defword(vm : &mut Vmem){
	if let (Some(Obj::S(_as)), Some(Obj::S(_bs))) = (vm.st.pop(), vm.st.pop()) {
		vm.ws.insert(_as, _bs);
	}
}
fn sayword(vm : &mut Vmem){
	if let Some(Obj::S(w)) = vm.st.pop() {
		let s = Obj::S(if let Some(wd) = vm.ws.get(&w)
			{ wd.clone() } else { w });
		vm.st.push(s)
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
pub static VMPRELUDE : &'static str = r#"[0 $]@popx \
[1 popx]@pop \
[1 1 $]@dupnth \
[1 dupnth]@dup \
[2 dupnth]@over \
[2 1 2 2 $]@dup2 \
[3 2 1 3 3 $]@dup3 \
[1 2 4 2 $]@swap \
[1 3 2 6 3 $]@rsh3 \
[2 1 3 6 3 $]@lsh3 \
[? .]@if \
[' rsh3 if]@iff \
[-1 *]@neg \
[1 0 lsh3 ?]@not \
[0 1 lsh3 ?]@boo \
[dup2 gt rsh3 - | boo]@gte \
[gt not]@lte \
[gte not]@lt \
[dup2 - not]@eq \
[dup2 - boo]@ne \
[dup =proc . ["proc while] iff]@while \
['while swap iff]@ifwhile \
[=proc dup =arr dup len =ln 0 [dup =i nth "proc . "i 1 + dup "ln gte] 0 "ln gt ifwhile]@map \
[print 10 prchr]@prln"#;
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
	b.insert("mka", mka);
	b.insert("siphon", siphon);
	b.insert("apush", pusha);
	b.insert("apop", popa);
	b.insert("nth", nth);
	b.insert("nthset", nthset);
	b.insert("len", len);
	b.insert("gt", gt);
	b.insert("set", setvar);
	b.insert("get", getvar);
	b.insert("t?", gettype);
	b.insert(".", execstr);
	b.insert("sayword", sayword);
	b.insert("defword", defword);
	Vmem { st: Vec::new(), ffi: b, vars: Vec::new(), ws: HashMap::new(), uop: None }
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
	vm.vars.push(HashMap::new());
	loop {
		let (opi, opend) = tokenize(code, opinext);
		if opend == 0 { break }
		opinext = opend;
		let op = &code[opi..opend];
		match op {
			"<@>" => vm.st.push(Obj::S(String::from(code))),
			"@>" => vm.st.push(Obj::S(String::from(&code[opi..]))),
			"<@" => vm.st.push(Obj::S(String::from(&code[..opi]))),
			"ret" => break,
			"err" => vm.st.push(Obj::E),
			_ if op.starts_with("'") => vm.st.push(Obj::S(String::from(&op[1..]))),
			_ if op.starts_with("\"") => {
				vm.st.push(Obj::S(String::from(&op[1..])));
				getvar(vm)
			},
			_ if op.starts_with("=") => {
				vm.st.push(Obj::S(String::from(&op[1..])));
				setvar(vm)
			},
			_ if op.starts_with("@") && op.len()>1 => {
				vm.st.push(Obj::S(String::from(&op[1..])));
				defword(vm)
			},
			_ if op.starts_with("[") => vm.st.push(Obj::S(parsestring(&op[1..op.len()-1]))),
			_ => execword(op, vm)
		}
	}
	vm.vars.pop();
}
