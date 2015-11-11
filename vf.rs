use std::char;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{Read,stdin};
use std::iter::Iterator;
use std::vec::*;
use vm::*;

fn ordobji(ord: Ordering) -> Obj {
	Obj::I(match ord {
		Ordering::Less => -1,
		Ordering::Equal => 0,
		Ordering::Greater => 1
	})
}

fn cmp(vm: &mut Vmem){
	if let (Some(bo),Some(ao)) = (vm.st.pop(), vm.st.pop())
		{ vm.st.push(ordobji(ao.cmp(&bo))) } else { vm.st.push(Obj::E) }
}
fn add(vm: &mut Vmem){
	if let (Some(bo),Some(ao)) = (vm.st.pop(), vm.st.pop())
		{ vm.st.push(iaddobj(ao, bo)) } else { vm.st.push(Obj::E) }
}
fn sub(vm: &mut Vmem){
	if let (Some(Obj::I(b)),Some(Obj::I(a))) = (vm.st.pop(), vm.st.pop())
		{ vm.st.push(Obj::I(a - b)) } else { vm.st.push(Obj::E) }
}
fn mul(vm: &mut Vmem){
	if let (Some(Obj::I(b)),Some(Obj::I(a))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(a * b))
	} else { vm.st.push(Obj::E) }
}
fn divmod(vm: &mut Vmem){
	if let (Some(Obj::I(b)),Some(Obj::I(a))) = (vm.st.pop(), vm.st.pop()){
		if b != 0{
			vm.st.push(Obj::I(a/b));
			vm.st.push(Obj::I(a%b))
		} else { vm.st.push(Obj::E) }
	}
}
fn band(vm: &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(a & b))
	}
}
fn bor(vm: &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(a | b))
	}
}
fn bxor(vm: &mut Vmem){
	if let (Some(Obj::I(a)),Some(Obj::I(b))) = (vm.st.pop(), vm.st.pop()){
		vm.st.push(Obj::I(a ^ b))
	}
}
fn pick(vm: &mut Vmem){
	if let Some(Obj::I(top)) = vm.st.pop() {
		if top == 0 { vm.st.pop(); }
		else {
			let len = vm.st.len();
			if len > 1 { vm.st.swap_remove(len-2); }
		}
	}
}
fn sform(vm: &mut Vmem){
	let slen = vm.st.len();
	if slen < 2 { return vm.st.clear() }
	if let (Obj::I(popxi), Obj::I(basei)) = (vm.st[slen-1].clone(), vm.st[slen-2].clone()) {
		if popxi < 0 || basei < 0 { return }
		let (popx, base) = (popxi as usize, basei as usize);
		if popx+2 > slen || base+2 > slen { return }
		let (spopx, sbase): (usize, usize) = (slen-popx-2, slen-base-2);
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
fn printobj(vm: &mut Vmem){
	if let Some(o) = vm.st.pop()
		{ print!("{}", objstr(&o)) }
		else { println!("Stack underflow") }
}
fn printchr(vm: &mut Vmem){
	if let Some(Obj::I(ai)) = vm.st.pop() {
		print!("{}", u32char(ai as u32))
	}
}
fn pushdepth(vm: &mut Vmem){
	let len = vm.st.len() as i64;
	vm.st.push(Obj::I(len));
}
fn mka(vm: &mut Vmem){
	vm.st.push(Obj::A(Vec::new()))
}
fn nth(vm: &mut Vmem){
	if let Some(Obj::I(n)) = vm.st.pop(){
		let r: Option<Obj> = match vm.st.last() {
			Some(&Obj::S(ref s)) => s.chars().nth(n as usize).map(|x| Obj::I(x as i64)),
			Some(&Obj::A(ref a)) => a.get(n as usize).map(|x| x.clone()),
			_ => None
		};
		vm.st.push(r.unwrap_or(Obj::E))
	}
}
fn siphon(vm: &mut Vmem) {
	let n = if let Some(Obj::I(n)) = vm.st.pop() { n } else { 0 };
	for _ in 0..n {
		let len = vm.st.len();
		vm.st.swap(len-2, len-1);
		pusha(vm)
	}
}
fn pusha(vm: &mut Vmem){
	if let Some(o) = vm.st.pop() {
		if let Some(&mut Obj::A(ref mut a)) = vm.st.last_mut() {
			a.push(o)
		}
	}
}
fn popa(vm: &mut Vmem){
	let ap = match vm.st.last_mut() {
		Some(&mut Obj::S(ref mut s)) => s.pop().map(|c| Obj::I(c as i64)),
		Some(&mut Obj::A(ref mut a)) => a.pop(),
		_ => None
	};
	vm.st.push(ap.unwrap_or(Obj::E))
}
fn nthset(vm: &mut Vmem){
	if let Some(Obj::I(idx)) = vm.st.pop() {
		if let Some(o) = vm.st.pop() {
			if let Some(&mut Obj::A(ref mut a)) = vm.st.last_mut() {
				a[idx as usize] = o
			}
		}
	}
}
fn len(vm: &mut Vmem){
	if let Some(o) = vm.st.pop(){
		vm.st.push(Obj::I(match o {
			Obj::S(s) => s.len() as i64,
			Obj::A(a) => a.len() as i64,
			_ => -1
		}));
	}
}
fn setvar(vm: &mut Vmem){
	if let (Some(Obj::S(s)),Some(o)) = (vm.st.pop(),vm.st.pop()) {
		if let Some(mut var) = vm.vars.last_mut() {
			var.insert(s, o);
		}
	}
}
fn getvar(vm: &mut Vmem){
	if let Some(Obj::S(s)) = vm.st.pop() {
		for vars in vm.vars.iter().rev() {
			if let Some(o) = vars.get(&s) {
				return vm.st.push(o.clone())
			}
		}
	}
	vm.st.push(Obj::E)
}
fn gettype(vm: &mut Vmem){
	let t = Obj::I(match vm.st.pop() {
		Some(Obj::E) => 0,
		Some(Obj::I(_)) => 1,
		Some(Obj::S(_)) => 2,
		Some(Obj::A(_)) => 3,
		None => -1
	});
	vm.st.push(t)
}
fn defword(vm: &mut Vmem){
	if let (Some(Obj::S(_as)), Some(Obj::S(_bs))) = (vm.st.pop(), vm.st.pop()) {
		vm.ws.insert(_as, _bs);
	}
}
fn sayword(vm: &mut Vmem){
	if let Some(Obj::S(w)) = vm.st.pop() {
		let s = Obj::S(if let Some(wd) = vm.ws.get(&w)
			{ wd.clone() } else { w });
		vm.st.push(s)
	}
}
fn execstr(vm: &mut Vmem){
	if let Some(Obj::S(code)) = vm.st.pop() {
		vmexec(vm, &code)
	}
}
fn getline(vm: &mut Vmem){
	let mut s = String::new();
	if let Ok(_) = stdin().read_line(&mut s) { vm.st.push(Obj::S(s)) }
}
fn execword(op: &str, vm: &mut Vmem){
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
			vmexec(vm, &wc)
		}else{
			if let Some(uop) = vm.uop {
				uop(vm, op)
			}
		}
	}
}
pub static VMPRELUDE: &'static str = r#"[0 $]@dropx \
[1 dropx]@drop \
[1 1 $]@dupnth \
[1 dupnth]@dup \
[2 dupnth]@over \
[2 1 2 2 $]@dup2 \
[3 2 1 3 3 $]@dup3 \
[1 2 4 2 $]@swap \
[1 3 1 $]@nip \
[1 2 1 5 3 $]@tuck \
[1 3 2 6 3 $]@rsh3 \
[2 1 3 6 3 $]@lsh3 \
[? .]@if \
[' rsh3 if]@iff \
[swap -]@minus \
[-1 *]@neg \
[1 0 lsh3 ?]@not \
[0 1 lsh3 ?]@boo \
[<=> not]@eq \
[<=> boo]@neq \
[<=> 1 eq]@gt \
[<=> -1 eq]@lt \
[<=> -1 neq]@gte \
[<=> 1 neq]@lte \
[dup =proc . ["proc while] iff]@while \
['while swap iff]@ifwhile \
[=proc dup =arr dup len =ln 0 [dup =i nth "proc . "i 1 + dup "ln gte] 0 "ln gt ifwhile]@map \
[print 10 prchr]@prln"#;
pub fn forthify(b: &mut HashMap<&'static str, fn(&mut Vmem)>) {
	b.insert("$", sform);
	b.insert("?", pick);
	b.insert("<=>", cmp);
	b.insert("+", add);
	b.insert("-", sub);
	b.insert("*", mul);
	b.insert("&", band);
	b.insert("|", bor);
	b.insert("^", bxor);
	b.insert("%/", divmod);
	b.insert("getline", getline);
	b.insert("print", printobj);
	b.insert("prchr", printchr);
	b.insert("depth", pushdepth);
	b.insert("mka", mka);
	b.insert("siphon", siphon);
	b.insert("push", pusha);
	b.insert("pop", popa);
	b.insert("nth", nth);
	b.insert("nthset", nthset);
	b.insert("len", len);
	b.insert("set", setvar);
	b.insert("get", getvar);
	b.insert("t?", gettype);
	b.insert(".", execstr);
	b.insert("sayword", sayword);
	b.insert("defword", defword);
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
pub fn vmexec(vm: &mut Vmem, code: &str){
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
