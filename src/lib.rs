use std::process::Command;
use interprocess::local_socket;
use std::io::prelude::*;
use std::collections::HashMap;
use json::JsonValue;
use std::fmt;
use std::process::Child;
#[cfg(windows)]
use dirs_next::home_dir;

const SOCKET_VAR: &str = "CAAT_SOCKET";
const ARGS_VAR: &str = "CAAT_ARGS";


#[derive(PartialEq, Clone)]
pub enum Value {
    Integer(i64),
    String(String),
    Float(f64),
    Dictionary(Vec<(String, Value)>),
    List(Box<[Value]>),
    Boolean(bool),
    Null,
}

impl Value {
    pub fn to_json(&self) -> String {
        match self {
            Value::Integer(i) => i.to_string(),
            Value::String(s) => format!("\"{}\"", s.clone()),
            Value::Float(f) => f.to_string(),
            Value::Dictionary(d) => {
                let mut result = String::from("{");
                for (key, value) in d {
                    result.push_str(&format!("\"{}\": {}, ", key, value.to_json()));
                }
                result.pop();
                result.pop();
                result.push_str("}");
                result
            }
            Value::List(l) => {
                let mut result = String::from("[");
                for value in l.into_iter() {
                    result.push_str(&format!("{}, ", value.to_json()));
                }
                result.pop();
                result.pop();
                result.push_str("]");
                result
            }
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
                    
        }
    }

    pub fn as_json(value: &[Value]) -> String {
        let mut result = String::from("[");
        for v in value {
            result.push_str(&v.to_json());
            result.push_str(", ");
        }

        result.pop();
        result.pop();
        
        result.push_str("]");
        return result;
    }
    
    pub fn from_json_value(value: JsonValue) -> Value {
        match value {
            JsonValue::Number(n) => {
                let temp: f64 = n.into();
                if temp % 1.0 != 0.0f64 {
                    Value::Float(n.into())
                } else {
                    Value::Integer(n.as_fixed_point_i64(0).expect("Number was not an integer"))
                }
            }
            JsonValue::String(s) => {
                let int = s.parse::<i64>();
                let float = s.parse::<f64>();
                if let Ok(i) = int {
                    Value::Integer(i)
                } else if let Ok(f) = float {
                    Value::Float(f)
                } else {
                    Value::String(s)
                }
            },
            JsonValue::Object(o) => {
                let mut map = Vec::new();
                for (key, value) in o.iter() {
                    map.push((key.to_string(), Value::from_json_value(value.clone())));
                }
                Value::Dictionary(map)
            }
            JsonValue::Array(a) => {
                let mut list = Vec::new();
                for value in a.iter() {
                    list.push(Value::from_json_value(value.clone()));
                }
                Value::List(list.into_boxed_slice())  
            }
            JsonValue::Boolean(b) => Value::Boolean(b),
            JsonValue::Null => Value::Null,
            JsonValue::Short(_) => panic!(),
        }
            
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "{}", s),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Dictionary(d) => {
                write!(f, "{{")?;
                for (key, value) in d {
                    write!(f, "\"{}\": {}, ", key, value)?;
                }
                write!(f, "}}")
            }
            Value::List(l) => {
                write!(f, "[")?;
                for value in l.iter() {
                    write!(f, "{}, ", value)?;
                }
                write!(f, "]")
            }
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "Integer({})", i),
            Value::String(s) => write!(f, "String({})", s),
            Value::Float(fl) => write!(f, "Float({})", fl),
            Value::Dictionary(d) => {
                write!(f, "Dictionary(")?;
                for (key, value) in d {
                    write!(f, "\"{}\": {}, ", key, value)?;
                }
                write!(f, ")")
            }
            Value::List(l) => {
                write!(f, "List(")?;
                for value in l.iter() {
                    write!(f, "{}, ", value)?;
                }
                write!(f, ")")
            }
            Value::Boolean(b) => write!(f, "Boolean({})", b),
            Value::Null => write!(f, "Null"),
        }
    }
}

impl From<u8> for Value {
    fn from(i: u8) -> Self {
        Value::Integer(i as i64)
    }
}

impl From<u16> for Value {
    fn from(i: u16) -> Self {
        Value::Integer(i as i64)
    }
}

impl From<u32> for Value {
    fn from(i: u32) -> Self {
        Value::Integer(i as i64)
    }
}

impl From<u64> for Value {
    fn from(i: u64) -> Self {
        Value::Integer(i as i64)
    }
}

impl From<i8> for Value {
    fn from(i: i8) -> Self {
        Value::Integer(i as i64)
    }
}

impl From<i16> for Value {
    fn from(i: i16) -> Self {
        Value::Integer(i as i64)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Integer(i as i64)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Integer(i)
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::Float(f as f64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Null
    }
}

impl From<Vec<(String, Value)>> for Value {
    fn from(d: Vec<(String, Value)>) -> Self {
        Value::Dictionary(d)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(d: HashMap<String, Value>) -> Self {
        Value::Dictionary(d.into_iter().collect())
    }
}

impl From<Box<[Value]>> for Value {
    fn from(l: Box<[Value]>) -> Self {
        Value::List(l)
    }
}

impl From<Vec<Value>> for Value {
    fn from(l: Vec<Value>) -> Self {
        Value::List(l.into_boxed_slice())
    }
}

impl TryFrom<Value> for u8 {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(i) => {
                if i < 0 || i > u8::MAX as i64 {
                    Err("Value is out of range")
                } else {
                    Ok(i as u8)
                }
            }
            _ => Err("Value is not an integer"),
        }
    }
}

impl TryFrom<Value> for u16 {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(i) => {
                if i < 0 || i > u16::MAX as i64 {
                    Err("Value is out of range")
                } else {
                    Ok(i as u16)
                }
            }
            _ => Err("Value is not an integer"),
        }
    }
}

impl TryFrom<Value> for u32 {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(i) => {
                if i < 0 || i > u32::MAX as i64 {
                    Err("Value is out of range")
                } else {
                    Ok(i as u32)
                }
            }
            _ => Err("Value is not an integer"),
        }
    }
}

impl TryFrom<Value> for u64 {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(i) => {
                if i < 0 {
                    Err("Value is out of range")
                } else {
                    Ok(i as u64)
                }
            }
            _ => Err("Value is not an integer"),
        }
    }
}

impl TryFrom<Value> for i8 {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(i) => {
                if i < i8::MIN as i64 || i > i8::MAX as i64 {
                    Err("Value is out of range")
                } else {
                    Ok(i as i8)
                }
            }
            _ => Err("Value is not an integer"),
        }
    }
}

impl TryFrom<Value> for i16 {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(i) => {
                if i < i16::MIN as i64 || i > i16::MAX as i64 {
                    Err("Value is out of range")
                } else {
                    Ok(i as i16)
                }
            }
            _ => Err("Value is not an integer"),
        }
    }
}

impl TryFrom<Value> for i32 {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(i) => {
                if i < i32::MIN as i64 || i > i32::MAX as i64 {
                    Err("Value is out of range")
                } else {
                    Ok(i as i32)
                }
            }
            _ => Err("Value is not an integer"),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(i) => Ok(i),
            _ => Err("Value is not an integer"),
        }
    }
}

impl TryFrom<Value> for f32 {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Float(f) => {
                if f < f32::MIN as f64 || f > f32::MAX as f64 {
                    Err("Value is out of range")
                } else {
                    Ok(f as f32)
                }
            }
            _ => Err("Value is not a float"),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Float(f) => Ok(f),
            _ => Err("Value is not a float"),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err("Value is not a string"),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(b) => Ok(b),
            _ => Err("Value is not a boolean"),
        }
    }
}

impl TryFrom<Value> for Vec<(String, Value)> {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Dictionary(d) => Ok(d),
            _ => Err("Value is not a dictionary"),
        }
    }
}

impl TryFrom<Value> for Box<[Value]> {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::List(l) => Ok(l),
            _ => Err("Value is not a list"),
        }
    }
}

impl TryFrom<Value> for Vec<Value> {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::List(l) => Ok(l.into_vec()),
            _ => Err("Value is not a list"),
        }
    }
}

impl TryFrom<Value> for () {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Null => Ok(()),
            _ => Err("Value is not null"),
        }
    }
}

impl TryFrom<Value> for HashMap<String, Value> {
    type Error = &'static str;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Dictionary(d) => Ok(d.into_iter().collect()),
            _ => Err("Value is not a dictionary"),
        }
    }
}


pub struct ForeignFunction<'a> {
    pub name: &'a str,
    args: Vec<&'a str>
}


impl<'a> ForeignFunction<'a> {
    pub fn new<S: ?Sized>(name: &'a S) -> ForeignFunction<'a> 
    where S: AsRef<str> {
    let split = name.as_ref().split_whitespace().collect::<Vec<&str>>();
    
    Self {
            name: split[0],
            args: split[1..].to_vec(),
        }
    }
}

impl ForeignFunction<'_> {
    pub fn call(&self, args: &[Value]) -> Value {
        let mut command = Command::new(self.name);
        for arg in &self.args {
            command.arg(arg);
        }
        for arg in args {
            command.arg(&arg.to_json());
        }
        let json = Value::as_json(args);

        command.env(ARGS_VAR, &json);
        let pid = std::process::id();
        #[cfg(unix)]
        let socket_path = format!("/tmp/caat_{}.sock", pid);
        #[cfg(windows)]
        let socket_path = format!("{}\\AppData\\Local\\Temp\\caat_{}.sock", home_dir(), pid);
        command.env(SOCKET_VAR, &socket_path);
        let handle = command.spawn().unwrap();

        return ForeignFunction::open_socket(handle, &socket_path);
    }

    #[inline]
    fn open_socket(mut handle: Child, socket_path: &str) -> Value {

        let listener = local_socket::LocalSocketListener::bind(socket_path).expect("Could not bind to socket");
        listener.set_nonblocking(true).expect("Could not set nonblocking");


        let mut stream = loop {
            match listener.accept() {
                Ok(stream) => break stream,
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        panic!("Error accepting connection");
                    }
                }
            }
        };

        let mut json_string = String::new();
        let mut buffer = [0; 1024];
        loop {
            match handle.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        return match status.code() {
                            Some(code) => Value::Integer(code as i64),
                            None => Value::Null,
                        };
                    }
                    break;
                },
                Ok(None) => (),
                Err(e) => panic!("Error waiting for process: {}", e),
            }
            let bytes = stream.read(&mut buffer).unwrap();
            json_string.push_str(&String::from_utf8_lossy(&buffer[..bytes]));
            if bytes == 1024 {
                break;
            }
        }
        let json = ForeignFunction::read_json(json_string);
        let _ = handle.wait();

        drop(stream);
        std::fs::remove_file(socket_path).unwrap();

        Value::from_json_value(json)
    }

    #[inline]
    fn read_json(string: String) -> JsonValue {
        return string.into()
    }
}

pub struct Args {
    args: Vec<Value>,
}

impl Args {
    pub fn from_json(json: JsonValue) -> Args {
        let mut args = Vec::new();
        for value in json.members() {
            args.push(Value::from_json_value(value.clone()));
        }
        Args { args }
    }

    pub fn from_args() -> Args {
        let mut args = Vec::new();
        for arg in std::env::args() {
            args.push(Value::String(arg));
        }
        Args { args }
    }
}


impl Iterator for Args {
    type Item = Value;
    fn next(&mut self) -> Option<Value> {
        self.args.drain(..1).next()
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<Value> {
        self.args.pop()
    }

    fn nth_back(&mut self, n: usize) -> Option<Value> {
        let mut back = self.args.pop();
        for _ in 0..(n -1) {
            back = self.args.pop();
        }
        back
    }
}

pub fn args() -> Args {
    match std::env::var(ARGS_VAR) {
        Ok(s) => {
            let json = s.into();
            Args::from_json(json)

        },
        Err(_) => Args::from_args(),
    }

}


#[macro_export]
macro_rules! return_caat {
    ($e:expr) => {
        let json = $e.into();
        let socket_path = std::env::var(SOCKET_VAR).unwrap();
        let mut stream = std::os::unix::net::UnixStream::connect(&socket_path).unwrap();
        stream.write_all(json.dump().as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Both).unwrap();
        std::process::exit(0);
    };
}



