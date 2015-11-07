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
