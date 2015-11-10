use std::char;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{Read,stdin};
use std::iter::Iterator;
use std::mem;
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
fn car(vm: &mut Vmem){
	vm.st.truncate(1)
}
fn cdr(vm: &mut Vmem){
	if vm.st.len() > 0 {
		vm.st.remove(0);
	}
}
fn add(vm: &mut Vmem){
	let ln = vm.st.len();
	if ln > 0 {
		let mut a = vm.st.swap_remove(0);
		let mut i = 1;
		while i < ln-i {
			let o = vm.st.swap_remove(i);
			a = iaddobj(a, o);
			i += 1
		}
		while ln-i > 0 {
			let o = vm.st.pop().unwrap();
			a = iaddobj(a, o);
			i += 1
		}
		vm.st.push(a)
	}
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
fn printobj(vm: &mut Vmem){
	while let Some(o) = vm.st.pop() {
		print!("{}", objstr(&o));
	}
}
fn u32char(u: u32) -> char{
	char::from_u32(u).unwrap_or('\u{fffd}')
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
fn getline(vm: &mut Vmem){
	let mut s = String::new();
	if let Ok(_) = stdin().read_line(&mut s) { vm.st.push(Obj::S(s)) }
}
pub static VMPRELUDE: &'static str = r#"(
(= fn (' n ...a f) (' = n (\a f))
(fn iff (' x y) (' (if x y (')))
(fn neg 'x (' (* x -1)))
(fn not 'x (' (if x 0 1)))
(fn boo 'x (' (if x 1 0)))
(fn eq (' x y) (' (not (<=> x y))))
(fn neq (' x y) (' (boo (<=> x y))))
(fn gt (' x y) (' (eq (<=> x y) 1)))
(fn lt (' x y) (' (eq (<=> x y) -1)))
(fn gte (' x y) (' (neq (<=> x y) -1)))
(fn lte (' x y) (' (neq (<=> x y) 1)))
(fn prln 'x (' (print x) (print 10))))"#;
fn xdigit(c: u32) -> u32 {
	if c >= ('0' as u32) && c <= ('9' as u32) { c-('0' as u32) }
	else if c >= ('a' as u32) && c<= ('z' as u32) { c-('a' as u32)+10 }
	else { 16 }
}
fn parsestring(s: &str) -> String{
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
				pm -= 1
			}
		}
	}
	idx
}
pub fn lispify(b: &mut HashMap<&'static str, fn(&mut Vmem)>) {
	b.insert("(+", add);
	b.insert("(-", sub);
	b.insert("(*", mul);
	b.insert("(%/", divmod);
	b.insert("(if", pick);
	b.insert("(<=>", cmp);
	b.insert("(print", printobj);
	b.insert("(nth", nth);
	b.insert("(car", car);
	b.insert("(cdr", cdr);
	//b.insert("(cons", cons);
}
// (+ (* 2 3 (+ 4 3)) (- 5 6))
// ["+", ["*", 2, 3, ["+", 4, 3]], ["-", 5 6]]
pub fn vmcompile(code: &str) -> Vec<Obj>{
	let mut smode = 0;
	let mut curstr = String::new();
	let mut lpos: usize = 0;
	let mut curls: Vec<Vec<Obj>> = Vec::new();
	let mut cval: Vec<Obj> = Vec::new();
	fn lparse(curls: &mut Vec<Vec<Obj>>, code: &str) {
		if let Some(ref mut curl) = curls.last_mut() {
			curl.push(if let Ok(val) = code.parse::<i64>()
				{ Obj::I(val) }else{ Obj::S(String::from(code)) });
		}
	};
	for (ci,c) in code.char_indices() {
		if lpos == 0 { lpos = ci }
		if smode == 0 {
			match c {
				'(' => {
					lparse(&mut curls, &code[lpos..ci]);
					lpos = 0;
					curls.push(Vec::new())
				},
				')' => {
					lparse(&mut curls, &code[lpos..ci]);
					lpos = 0;
					if let Some(l) = curls.pop() {
						if let Some(ll) = curls.last_mut() {
							ll.push(Obj::A(l))
						} else {
							cval = l;
							break
						}
					} else { break }
				},
				'[' => {
					lparse(&mut curls, &code[lpos..ci]);
					lpos = 0;
					smode = 1
				},
				' ' => {
					lparse(&mut curls, &code[lpos..ci]);
					lpos = 0
				},
				_ => ()
			}
		}else{
			if c == '[' { smode +=1 }
			else if c == ']' {
				smode -=1;
				if smode == 0 {
					curls.last_mut().unwrap().push(Obj::S(curstr.clone()));
					curstr.clear();
					continue
				}
			}
			curstr.push(c)
		}
	}
	cval
}
pub fn vmeval(vm: &mut Vmem, code: Vec<Obj>) -> Obj {
	let mut codev = Vec::new();
	for o in code {
		if let Obj::A(expr) = o
			{ codev.push(vmeval(vm, expr)) } else { codev.push(o) }
	}
	let mut code = codev.into_iter();
	if let Some(Obj::S(op)) = code.next() {
		let mut preop = String::from("(");
		preop.push_str(&op);
		let fc = if let Some(func) = vm.ffi.get(&preop[..])
			{ Some(func.clone()) } else { None };
		if let Some(fc) = fc {
			vm.st = code.collect();
			fc(vm)
		}
	}
	Obj::A(mem::replace(&mut vm.st, Vec::new()))
}
pub fn vmexec(vm: &mut Vmem, code: &str){
	vmeval(vm, vmcompile(code));
}
