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
fn mapop(vm: &mut Vmem, f: fn(Obj) -> Obj) {
	let ost = mem::replace(&mut vm.st, Vec::new());
	vm.st.extend(ost.into_iter().map(f))
}
fn binop(vm: &mut Vmem, f: fn(Obj, Obj) -> Obj) {
	let ln = vm.st.len();
	if ln > 0 {
		let mut a = vm.st.swap_remove(0);
		let mut i = 1;
		while i < ln-i {
			a = f(a, vm.st.swap_remove(i));
			i += 1
		}
		while ln-i > 0 {
			a = f(a, vm.st.pop().unwrap());
			i += 1
		}
		vm.st.push(a)
	}
}
fn add(vm: &mut Vmem){
	binop(vm, iaddobj)
}
fn sub(vm: &mut Vmem){
	fn func(a: Obj, b: Obj) -> Obj {
		if let (Obj::I(a), Obj::I(b)) = (a,b)
			{ Obj::I(a-b) } else { Obj::E }
	}
	binop(vm, func)
}
fn mul(vm: &mut Vmem){
	fn func(a: Obj, b: Obj) -> Obj {
		if let (Obj::I(a), Obj::I(b)) = (a,b)
			{ Obj::I(a*b) } else { Obj::E }
	}
	binop(vm, func)
}
fn opdiv(vm: &mut Vmem){
	fn func(a: Obj, b: Obj) -> Obj {
		if let (Obj::I(a), Obj::I(b)) = (a,b)
			{ if b != 0 { Obj::I(a/b) } else { Obj::E } } else { Obj::E }
	}
	binop(vm, func)
}
fn opmod(vm: &mut Vmem){
	fn func(a: Obj, b: Obj) -> Obj {
		if let (Obj::I(a), Obj::I(b)) = (a,b)
			{ if b != 0 { Obj::I(a%b) } else { Obj::E } } else { Obj::E }
	}
	binop(vm, func)
}
fn band(vm: &mut Vmem){
	fn func(a: Obj, b: Obj) -> Obj {
		if let (Obj::I(a), Obj::I(b)) = (a,b)
			{ Obj::I(a&b) } else { Obj::E }
	}
	binop(vm, func)
}
fn bor(vm: &mut Vmem){
	fn func(a: Obj, b: Obj) -> Obj {
		if let (Obj::I(a), Obj::I(b)) = (a,b)
			{ Obj::I(a|b) } else { Obj::E }
	}
	binop(vm, func)
}
fn bxor(vm: &mut Vmem){
	fn func(a: Obj, b: Obj) -> Obj {
		if let (Obj::I(a), Obj::I(b)) = (a,b)
			{ Obj::I(a^b) } else { Obj::E }
	}
	binop(vm, func)
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
fn concat(vm: &mut Vmem){
	let mut nst = mem::replace(&mut vm.st, Vec::new());
	for a in nst.into_iter() {
		if let Obj::A(a) = a {
			vm.st.extend(a.into_iter());
		}
	}
}
fn printobj(vm: &mut Vmem){
	while let Some(o) = vm.st.pop() {
		print!("{}", objstr(&o));
	}
}
fn printchr(vm: &mut Vmem){
	if let Some(Obj::I(ai)) = vm.st.pop() {
		print!("{}", u32char(ai as u32))
	}
}
fn qlen(vm: &mut Vmem){
	let len = vm.st.len() as i64;
	vm.st = vec![Obj::I(len)]
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
	fn f(o: Obj) -> Obj{
		Obj::I(match o {
			Obj::S(s) => s.len() as i64,
			Obj::A(a) => a.len() as i64,
			_ => -1
		})
	}
	mapop(vm, f)
}
fn quote(vm: &mut Vmem){
	let ost = mem::replace(&mut vm.st, Vec::new());
	vm.st.push(Obj::A(ost))
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
(= nil {})
(= fn {n ..a f} (' = n (\a f))
(fn void {..a} {})
(fn inline {..a} {"a})
(fn iff {x y} (' (if x y (')))
(fn neg x (' (* x -1)))
(fn not x (' (if x 0 1)))
(fn boo x (' (if x 1 0)))
(fn eq {x y} (' (not (<=> x y))))
(fn neq {x y} (' (boo (<=> x y))))
(fn gt {x y} (' (eq (<=> x y) 1)))
(fn lt {x y} (' (eq (<=> x y) -1)))
(fn gte {x y} (' (neq (<=> x y) -1)))
(fn lte {x y} (' (neq (<=> x y) 1)))
(fn prln x (' (print x) (print 10))))"#;
pub fn lispify(b: &mut HashMap<&'static str, fn(&mut Vmem)>) {
	b.insert("(+", add);
	b.insert("(-", sub);
	b.insert("(*", mul);
	b.insert("(/", opdiv);
	b.insert("(%", opmod);
	b.insert("(if", pick);
	b.insert("(<=>", cmp);
	b.insert("(concat", concat);
	b.insert("(print", printobj);
	b.insert("(len", len);
	b.insert("(nth", nth);
	b.insert("(=", setvar);
	b.insert("(QUOTE", quote);
	b.insert("(qlen", qlen);
}
pub fn vmcompile(code: &str) -> Vec<Obj>{
	let mut smode = 0;
	let mut wmode = true;
	let mut escmode = false;
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
		if smode == 0 {
			if wmode {
				if c.is_whitespace() { lpos = ci+c.len_utf8();continue } else { wmode = false }
			}
			match c {
				'{'|'}'|'('|')'|'[' => (),
				_ if c.is_whitespace() => (),
				_ => continue
			}
			lparse(&mut curls, &code[lpos..ci]);
			lpos = ci+c.len_utf8();
			match c {
				'{' => curls.push(vec![Obj::S(String::from("(QUOTE"))]),
				'(' => curls.push(Vec::new()),
				')'|'}' => {
					if let Some(l) = curls.pop() {
						if let Some(ll) = curls.last_mut() {
							ll.push(Obj::A(l))
						} else {
							cval = l;
							break
						}
					} else { break }
				},
				'[' => smode = 1,
				_ if c.is_whitespace() => wmode = true,
				_ => unreachable!()
			}
		}else{
			if escmode { escmode = false }
			else if c == '[' { smode +=1 }
			else if c == ']' {
				smode -=1;
				if smode == 0 {
					curls.last_mut().unwrap().push(Obj::S(parsestring(&code[lpos..ci])));
					continue
				}
			}else if c == '\\' { escmode = true }
		}
	}
	cval
}
pub fn vmeval(vm: &mut Vmem, code: Vec<Obj>) {
	let mut codev = Vec::new();
	for o in code {
		if let Obj::A(expr) = o {
			vmeval(vm, expr);
			codev.extend(mem::replace(&mut vm.st, Vec::new()).into_iter())
		} else { codev.push(o) }
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
}
pub fn vmexec(vm: &mut Vmem, code: &str) {
	vmeval(vm, vmcompile(code))
}
