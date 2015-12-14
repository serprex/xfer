use std::collections::hash_map::{Entry,HashMap};
use std::io::{Read,stdin};
use std::iter::Iterator;
use std::mem;
use std::vec::*;
use vm::*;

fn cmp(vm: &mut Vmem){
	if let (Some(bo),Some(ao)) = (vm.st.pop(), vm.st.pop())
		{ vm.st.push(ordobji(ao.cmp(&bo))) } else { vm.st.push(Obj::E) }
}
fn mapop(vm: &mut Vmem, f: fn(Obj) -> Obj) {
	let ost = mem::replace(&mut vm.st, Vec::new());
	vm.st.extend(ost.into_iter().map(f))
}
fn mappairs(vm: &mut Vmem, f: fn(Obj, Obj) -> Obj) {
	let mut oi = mem::replace(&mut vm.st, Vec::new()).into_iter();
	while let Some(a) = oi.next() {
		let b = oi.next().unwrap_or(Obj::E);
		vm.st.push(f(a, b))
	}
}
fn binop(vm: &mut Vmem, f: fn(Obj, Obj) -> Obj) {
	if vm.st.is_empty() { return }
	let mut oi = mem::replace(&mut vm.st, Vec::new()).into_iter();
	let mut a = oi.next().unwrap();
	for o in oi {
		a = f(a, o)
	}
	vm.st.push(a)
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
	let mut oi = mem::replace(&mut vm.st, Vec::new()).into_iter();
	if let Some(pred) = oi.next() {
		if let Some(truth) = oi.next() {
			match pred {
				Obj::I(0) => vm.st.extend(oi),
				_ => vm.st.push(truth)
			}
		}
	}
}
fn concat(vm: &mut Vmem){
	let ost = mem::replace(&mut vm.st, Vec::new()).into_iter();
	for o in ost {
		if let Obj::A(a) = o {
			vm.st.extend(a.into_iter());
		}
	}
}
fn printobj(vm: &mut Vmem){
	let ost = mem::replace(&mut vm.st, Vec::new()).into_iter();
	for o in ost {
		print!("{}", objstr(&o))
	}
}
fn printchr(vm: &mut Vmem){
	let ost = mem::replace(&mut vm.st, Vec::new()).into_iter();
	let mut s = String::new();
	for o in ost {
		if let Obj::I(i) = o {
			s.push(u32char(i as u32))
		}
	}
	print!("{}", s)
}
fn qlen(vm: &mut Vmem){
	let len = vm.st.len() as i64;
	vm.st = vec![Obj::I(len)]
}
fn nth(vm: &mut Vmem){
	fn f(a: Obj, b: Obj) -> Obj {
		if let Obj::I(n) = a{
			let r: Option<Obj> = match b {
				Obj::S(s) => s.chars().nth(n as usize).map(|x| Obj::I(x as i64)),
				Obj::A(mut a) => {
					let ln = a.len();
					let un = n as usize;
					Some(if un < ln { a.swap_remove(un) } else { Obj::E })
				},
				_ => None
			};
			r.unwrap_or(Obj::E)
		} else { Obj::E }
	}
	mappairs(vm, f)
}
fn subset(vm: &mut Vmem){
	let mut oi = mem::replace(&mut vm.st, Vec::new()).into_iter();
	if let (Some(Obj::I(mut start)), Some(Obj::I(mut end))) = (oi.next(), oi.next()){
		if start > end { mem::swap(&mut start, &mut end) }
		vm.st.extend(oi.into_iter().skip(start as usize).take((end-start) as usize))
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
fn deepquote(vm: &mut Vmem){
	let mut ost = mem::replace(&mut vm.st, Vec::new());
	ost.insert(0, Obj::S(String::from("\"")));
	vm.st.push(Obj::A(ost))
}
fn tail(vm: &mut Vmem){
	for o in &mut vm.st {
		if let &mut Obj::A(ref mut a) = o {
			if !a.is_empty() { a.remove(0); }
		}
	}
}
fn inline(_: &mut Vmem){
}
fn setvar(vm: &mut Vmem){
	if vm.st.is_empty() { return }
	let mut ost = mem::replace(&mut vm.st, Vec::new());
	if let Obj::S(s) = ost.remove(0) {
		if let Some(mut var) = vm.vars.last_mut() {
			var.insert(s, Obj::A(ost));
		}
	}
}
fn upvar(vm: &mut Vmem){
	if vm.st.is_empty() { return }
	let mut ost = mem::replace(&mut vm.st, Vec::new());
	if let Obj::S(s) = ost.remove(0) {
		let aost = Obj::A(ost);
		for vars in vm.vars.iter_mut().rev() {
			if let Entry::Occupied(mut e) = vars.entry(s.clone()) {
				e.insert(aost);
				return
			}
		}
		if let Some(vars) = vm.vars.first_mut() {
			vars.insert(s, aost);
		}
	}
}
fn getvar(vm: &mut Vmem){
	let ost = mem::replace(&mut vm.st, Vec::new()).into_iter();
	for o in ost {
		if let Obj::S(s) = o {
			for vars in vm.vars.iter().rev() {
				if let Some(v) = vars.get(&s) {
					if let &Obj::A(ref a) = v { vm.st.extend(a.clone()) }
					else { vm.st.push(v.clone()) }
					continue
				}
			}
		} else { vm.st.push(o) }
	}
}
fn gettype(vm: &mut Vmem){
	fn f(o: Obj) -> Obj {
		Obj::I(match o {
			Obj::E => 0,
			Obj::I(_) => 1,
			Obj::S(_) => 2,
			Obj::A(_) => 3
		})
	}
	mapop(vm, f)
}
fn getline(vm: &mut Vmem){
	let mut s = String::new();
	if let Ok(_) = stdin().read_line(&mut s) { vm.st = vec![Obj::S(s)] }
}
fn fcall(vm: &mut Vmem){
	let mut code = mem::replace(&mut vm.st, Vec::new()).into_iter();
	if let Some(Obj::A(mut op)) = code.next() {
		if let Some(Obj::A(opf)) = op.pop() {
			let mut scope = HashMap::new();
			let mut oi = op.into_iter();
			let mut isvari = None;
			while let Some(o) = oi.next() {
				if let Obj::S(s) = o {
					if s.starts_with("..") {
						isvari = Some(String::from(&s[2..]));
						break
					}else if let Some(co) = code.next() {
						println!("\t{}={}", s, objstr(&co));
						scope.insert(s, Obj::A(vec![co]));
					}else { break }
				}
			}
			while let Some(o) = oi.next_back() {
				if let Obj::S(s) = o {
					if let Some(co) = code.next_back() {
						scope.insert(s, Obj::A(vec![co]));
					}else { break }
				}
			}
			if let Some(vari) = isvari {
				println!("\t{}", vari);
				scope.insert(vari, Obj::A(code.collect()));
			}
			vm.vars.push(scope);
			vmeval(vm, opf);
			vm.vars.pop();
		}
	}
}
pub static VMPRELUDE: &'static str = r#"(inline
(#prefix " (" @))
(#prefix ' (' @))
(#prefix $ ($ @))
(#prefix ! ! $@)
(~ fn (tail <n ..a f (~ $n {$a (tail $f)})>))
(~ lfn (tail <n ..a f (= %n {$a (tail $f)}))>)
(!fn neg x <- 0 $x>)
(!fn prln ..a <(print $a) (prchr 10)>)
(!fn eval1 a <!{a}>)
(!fn eval ..a <!{{$a}}>)
(!fn not x <if $x 0 1>)
(!fn boo x <if $x 1 0>)
(!fn nil? ..a <!not (qlen $a)>)
(!fn eq x y <!not (cmp $x $y)>)
(!fn neq x y <!boo (cmp $x $y)>)
(!fn gt x y <!eq (cmp $x $y) 1>)
(!fn lt x y <!eq (cmp $x $y) -1>)
(!fn gte x y <!neq (cmp $x $y) -1>)
(!fn lte x y <!neq (cmp $x $y) 1>)
(!fn map f x ..a <concat {!f $x} (concat {! (tail (if (!nil? $a) {} <!map $f $a>))})>)
)"#;
pub fn lispify(b: &mut HashMap<&'static str, fn(&mut Vmem)>) {
	b.insert("(+", add);
	b.insert("(-", sub);
	b.insert("(*", mul);
	b.insert("(/", opdiv);
	b.insert("(%", opmod);
	b.insert("(&", band);
	b.insert("(|", bor);
	b.insert("(^", bxor);
	b.insert("(if", pick);
	b.insert("(cmp", cmp);
	b.insert("(concat", concat);
	b.insert("(tail", tail);
	b.insert("(print", printobj);
	b.insert("(prchr", printchr);
	b.insert("(getline", getline);
	b.insert("(len", len);
	b.insert("(nth", nth);
	b.insert("(slice", subset);
	b.insert("($", getvar);
	b.insert("(=", setvar);
	b.insert("(~", upvar);
	b.insert("('", quote);
	b.insert("(\"", deepquote);
	b.insert("(inline", inline);
	b.insert("(qlen", qlen);
	b.insert("(typeof", gettype);
	b.insert("(!", fcall);
}
pub fn vmcompile(code: &str, prefixes: &mut HashMap<char, Vec<Obj>>) -> Vec<Obj>{
	let mut smode = 0;
	let mut escmode = false;
	let mut lpos: usize = 0;
	let mut curls: Vec<Vec<Obj>> = Vec::new();
	let mut cval: Vec<Obj> = Vec::new();
	fn lparse(curls: &mut [Vec<Obj>], prefixes: &HashMap<char, Vec<Obj>>, code: &str) {
		if code.is_empty() { return }
		if let Some(ref mut curl) = curls.last_mut() {
			if let Ok(val) = code.parse::<i64>()
				{ curl.push(Obj::I(val)) }
			else {
				if code.chars().nth(1).is_some() {
					if let Some(pfcode) = prefixes.get(&code.chars().nth(0).unwrap()) {
						fn repat(a: &[Obj], at: &str) -> Vec<Obj> {
							a.iter().map(|o| {
								match o {
									&Obj::S(ref s) if s == "@" => Obj::S(String::from(at)),
									&Obj::A(ref a2) => Obj::A(repat(a2, at)),
									_ => o.clone()
								}
							}).collect()
						}
						return curl.extend(repat(pfcode, &code[code.chars().nth(0).unwrap().len_utf8()..]))
					}
				}
				curl.push(Obj::S(String::from(code)))
			}
		}
	};
	for (ci,c) in code.char_indices() {
		if smode == 0 {
			match c {
				'('|')'|'<'|'>'|'{'|'}'|'[' => (),
				_ if c.is_whitespace() => (),
				_ => continue
			}
			lparse(&mut curls, &prefixes, &code[lpos..ci]);
			lpos = ci+c.len_utf8();
			match c {
				'(' => curls.push(Vec::new()),
				'{' => curls.push(vec![Obj::S(String::from("'"))]),
				'<' => curls.push(vec![Obj::S(String::from("\""))]),
				')'|'}'|'>' => {
					if let Some(l) = curls.pop() {
						{	let mut li = l.iter();
							if let Some(&Obj::S(ref fop)) = li.next() {
								if fop == "#prefix" {
									if let Some(&Obj::S(ref prefix)) = li.next() {
										if let Some(ch) = prefix.chars().nth(0) {
											prefixes.insert(ch, li.map(|x| x.clone()).collect());
										}
									}
								}
							}
						}
						if let Some(ll) = curls.last_mut() {
							ll.push(Obj::A(l))
						} else {
							cval = l;
							break
						}
					} else { break }
				},
				'[' => smode = 1,
				_ if c.is_whitespace() => (),
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
	if if let Some(&Obj::S(ref s)) = code.first() {
		s == "\""
	} else { false } { return vm.st = vec![Obj::A(code)] }
	let mut codev = Vec::new();
	for o in code {
		if let Obj::A(expr) = o {
			vmeval(vm, expr);
			codev.extend(mem::replace(&mut vm.st, Vec::new()).into_iter())
		} else { codev.push(o) }
	}
	let mut code = codev.into_iter();
	if let Some(Obj::S(op)) = code.next() {
		if op == "\"" { return }
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
pub fn vmexec(vm: &mut Vmem, prefixes: &mut HashMap<char, Vec<Obj>>, code: &str) {
	vm.vars.push(HashMap::new());
	vmeval(vm, vmcompile(code, prefixes));
	vm.vars.pop();
}
