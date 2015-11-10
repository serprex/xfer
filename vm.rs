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
