use std::process::Command;
use interprocess::local_socket;
use std::io::prelude::*;
use std::collections::HashMap;
use json::JsonValue;
use std::fmt::{self, write};
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
    Map(Vec<(String, Value)>),
    List(Box<[Value]>),
    Boolean(bool),
    Null,
    CAATFunction(ForeignFunction),
}

impl Value {
    pub fn to_json(&self) -> String {
        match self {
            Value::Integer(i) => format!("{{\"type\": \"Integer\", \"value\": {}}}", i),
            Value::String(s) => format!("{{\"type\": \"String\", \"value\": \"{}\"}}", s),
            Value::Float(f) => format!("{{\"type\": \"Float\", \"value\": {}}}", f),
            Value::Map(d) => {
                let mut result = String::from("{");
                for (key, value) in d {
                    result.push_str(&format!("\"{}\": {}, ", key, value.to_json()));
                }
                result.pop();
                result.pop();
                result.push_str("}");
                format!("{{\"type\": \"Map\", \"value\": {}}}", result)
            }
            Value::List(l) => {
                let mut result = String::from("[");
                for value in l.into_iter() {
                    result.push_str(&format!("{}, ", value.to_json()));
                }
                result.pop();
                result.pop();
                result.push_str("]");
                format!("{{\"type\": \"List\", \"value\": {}}}", result)
            }
            Value::Boolean(b) => format!("{{\"type\": \"Boolean\", \"value\": {}}}", b),
            Value::Null => format!("{{\"type\": \"Null\", \"value\": null}}"),
            Value::CAATFunction(s) => format!("{{\"type\": \"CAAT\", \"value\": \"{}\"}}", s),
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
    
    pub fn from_json_value(value: &JsonValue) -> Option<Value> {
        eprintln!("{:?}", value);
        match value {
            JsonValue::Object(o) => {
                if let Some(value) = o.get("type") {
                    if let Some(the_type) = value.as_str() {
                        match the_type {
                            "Integer" => {
                                if let Some(value) = o.get("value") {
                                    if let Some(i) = value.as_i64() {
                                        return Some(Value::Integer(i));
                                    } else {
                                        return Some(Value::Integer(0));
                                    }
                                } else {
                                    return None;
                                }
                            },
                            "Float" => {
                                if let Some(value) = o.get("value") {
                                    if let Some(f) = value.as_f64() {
                                        return Some(Value::Float(f));
                                    } else {
                                        return Some(Value::Float(0.0));
                                    }
                                } else {
                                    return None;
                                }
                            },
                            "String" => {
                                if let Some(value) = o.get("value") {
                                    if let Some(s) = value.as_str() {
                                        return Some(Value::String(s.to_string()));
                                    } else {
                                        return Some(Value::String(String::new()));
                                    }
                                } else {
                                    return None;
                                }
                            },
                            "Map" => {
                                if let Some(value) = o.get("value") {
                                    match value {
                                        JsonValue::Object(o) => {
                                            let mut map = Vec::new();
                                            for (key, value) in o.iter() {
                                                map.push((key.to_string(), Value::from_json_value(value)?))
                                            }
                                            return Some(Value::Map(map));
                                        }
                                        _ => None
                                    }
                                } else {
                                    return None;
                                }
                            },
                            "List" => {
                                if let Some(value) = o.get("value") {
                                    match value {
                                        JsonValue::Array(a) => {
                                            let mut list = Vec::new();
                                            for value in a.iter() {
                                                list.push(Value::from_json_value(value)?);
                                            }
                                            Some(Value::List(list.into_boxed_slice()))
                                        }
                                        _ => None
                                    }
                                } else {
                                    return None;
                                }
                            },
                            "CAAT" => {
                                if let Some(value) = o.get("value") {
                                    if let Some(command) = value.as_str() {
                                        return Some(Value::CAATFunction(ForeignFunction::new(command)));
                                    } else {
                                        return None;
                                    }
                                } else {
                                    return None;
                                }
                            }
                            _ => None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None
        }
            
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Map(d) => {
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
            Value::CAATFunction(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "Integer({})", i),
            Value::String(s) => write!(f, "String({})", s),
            Value::Float(fl) => write!(f, "Float({})", fl),
            Value::Map(d) => {
                write!(f, "Map(")?;
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
            Value::CAATFunction(s) => write!(f, "Function({})", s),
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
        Value::Map(d)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(d: HashMap<String, Value>) -> Self {
        Value::Map(d.into_iter().collect())
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
            Value::Map(d) => Ok(d),
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
            Value::Map(d) => Ok(d.into_iter().collect()),
            _ => Err("Value is not a dictionary"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ForeignFunction {
    pub name: String,
    args: Vec<String>
}


impl ForeignFunction {
    pub fn new<S: ?Sized>(name: &S) -> ForeignFunction
    where S: AsRef<str> {
    let split = name.as_ref().split_whitespace().collect::<Vec<&str>>();
    
    Self {
            name: split[0].to_string(),
            args: split[1..].into_iter().map(|x| x.to_string()).collect(),
        }
    }
}

impl ForeignFunction {
    pub fn call(&self, args: &[Value]) -> Value {
        let mut command = Command::new(&self.name);
        let mut new_args = Vec::new();
        for arg in &self.args {
            command.arg(arg);
            new_args.push(Value::String(arg.to_string()));
        }
        for arg in args {
            match arg {
                Value::String(value) => command.arg(&value),
                _ => command.arg(&arg.to_json()),
            };
        }
        new_args.extend_from_slice(args);
        let json = Value::as_json(&new_args);

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

        let mut try_wait = 3;
        let mut stream = loop {
            match listener.accept() {
                Ok(stream) => break stream,
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        panic!("Error accepting connection");
                    }
                }
            }
            if try_wait < 3 {
                try_wait += 1;
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }

            match handle.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        
                        match listener.accept() {
                            Ok(stream) => break stream,
                            Err(e) => {
                                if e.kind() != std::io::ErrorKind::WouldBlock {
                                    panic!("Error accepting connection");
                                }
                            }
                        }
                        std::fs::remove_file(socket_path).unwrap();
                        return match status.code() {
                            Some(code) => Value::Integer(code as i64),
                            None => Value::Null,
                        };
                    } else {
                        match listener.accept() {
                            Ok(stream) => break stream,
                            Err(e) => {
                                if e.kind() != std::io::ErrorKind::WouldBlock {
                                    panic!("Error accepting connection");
                                }
                            }
                        }
                        std::fs::remove_file(socket_path).unwrap();
                        return match status.code() {
                            Some(code) => Value::Integer(code as i64),
                            None => Value::Null,
                        };
                    }
                },
                Ok(None) => (),
                Err(e) => panic!("Error waiting for process: {}", e),
            }
            try_wait = 0;
        };

        let mut json_string = String::new();
        let mut buffer = [0; 1024];
        loop {
            match handle.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        std::fs::remove_file(socket_path).unwrap();
                        return match status.code() {
                            Some(code) => Value::Integer(code as i64),
                            None => Value::Null,
                        };
                    } 
                },
                Ok(None) => (),
                Err(e) => panic!("Error waiting for process: {}", e),
            }
            let bytes = stream.read(&mut buffer).unwrap();
            json_string.push_str(&String::from_utf8_lossy(&buffer[..bytes]));
            if bytes < 1024 {
                break;
            }
        }
        let json = ForeignFunction::read_json(json_string);
        let _ = handle.wait();

        drop(stream);
        std::fs::remove_file(socket_path).unwrap();

        Value::from_json_value(&json).expect("Failed")
    }

    #[inline]
    fn read_json(string: String) -> JsonValue {
        return string.into()
    }
}

impl fmt::Display for ForeignFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.args.join(" "))
    }
}

pub struct Args {
    args: Vec<Value>,
}

impl Args {
    pub fn from_json(json: JsonValue) -> Args {
        let mut args = Vec::new();
        for value in json.members() {
            args.push(Value::from_json_value(value).unwrap());
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



