use std::char;
use std::collections::HashMap;
#[derive(Clone,PartialEq,Eq,PartialOrd,Ord)]
pub enum Obj{
	E,
	I(i64),
	S(String),
	A(Vec<Obj>),
}
#[derive(Default)]
pub struct Vmem{
	pub st: Vec<Obj>,
	pub vars: Vec<HashMap<String, Obj>>,
	pub ws: HashMap<String, String>,
	pub ffi: HashMap<&'static str, fn(&mut Vmem)>,
	pub uop: Option<fn(&mut Vmem, &str)>,
}
pub fn u32char(u: u32) -> char{
	char::from_u32(u).unwrap_or('\u{fffd}')
}
pub fn iaddobj(ao: Obj, bo: Obj) -> Obj{
	match (ao,bo) {
		(Obj::I(a),Obj::I(b)) => Obj::I(a + b),
		(Obj::S(mut a),Obj::S(b)) => {
			a.push_str(&b);
			Obj::S(a)
		},
		(Obj::A(mut a),Obj::A(b)) => {
			a.extend(b.iter().map(|x| x.clone()));
			Obj::A(a)
		},
		_ => Obj::E
	}
}
pub fn objstr(o: &Obj) -> String {
	match o {
		&Obj::E => String::from("E"),
		&Obj::I(ref x) => format!("{}", x),
		&Obj::S(ref x) => format!("{}", x),
		&Obj::A(ref x) => {
			let mut s = String::new();
			for a in x {
				s.push_str(&objstr(a));
				s.push(' ')
			}
			s.pop();
			format!("({})", s)
		}
	}
}
pub fn parsestring(s: &str) -> String{
	let mut ret = String::new();
	let mut esc = false;
	let mut hex = 0;
	let mut uni = 0;
	for c in s.chars() {
		if !esc && c == '\\'{
			esc = true
		} else {
			if esc {
				let c32 = c as u32;
				let xd = if c32 >= ('0' as u32) && c32 <= ('9' as u32) { c32-('0' as u32) }
					else if c32 >= ('a' as u32) && c32 <= ('z' as u32) { c32-('a' as u32)+10 }
					else { 16 };
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
