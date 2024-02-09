use std::process::Command;
use std::os::unix::net::UnixListener;
use std::io::prelude::*;

use json::JsonValue;


const SOCKET_VAR: &str = "CAAT_SOCKET";
const ARGS_VAR: &str = "CAAT_ARGS";


#[derive(Debug, PartialEq)]
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
            Value::String(s) => s.clone(),
            Value::Float(f) => f.to_string(),
            Value::Dictionary(d) => {
                let mut result = String::from("{");
                for (key, value) in d {
                    result.push_str(&format!("\"{}\": {}, ", key, value.to_json()));
                }
                result.push_str("}");
                result
            }
            Value::List(l) => {
                let mut result = String::from("[");
                for value in l.into_iter() {
                    result.push_str(&format!("{}, ", value.to_json()));
                }
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




pub struct ForeignFunction<'a> {
    pub name: &'a str,
}


impl<'a> ForeignFunction<'a> {
    pub fn new<S: ?Sized>(name: &'a S) -> ForeignFunction<'a> 
    where S: AsRef<str> {
    Self {
            name: name.as_ref(),
        }
    }
}

impl ForeignFunction<'_> {
    pub fn call(&self, args: &[Value]) -> Value {
        let mut command = Command::new(self.name);
        for arg in args {
            command.arg(&arg.to_json());
        }
        let json = Value::as_json(args);

        command.env(ARGS_VAR, &json);
        let pid = std::process::id();
        let socket_path = format!("/tmp/caat_{}.sock", pid);
        command.env(SOCKET_VAR, &socket_path);
        let mut handle = command.spawn().unwrap();

        let listener = UnixListener::bind(&socket_path).unwrap();
        let (mut stream, _) = listener.accept().unwrap();
        let mut json_string = String::new();
        let mut buffer = [0; 1024];
        loop {
            let bytes = stream.read(&mut buffer).unwrap();
            if bytes == 0 {
                break;
            }
            json_string.push_str(&String::from_utf8_lossy(&buffer[..bytes]));
        }
        let json = ForeignFunction::read_json(json_string);
        let _ = handle.wait();

        stream.shutdown(std::net::Shutdown::Both).unwrap();
        std::fs::remove_file(socket_path).unwrap();

        Value::from_json_value(json)



    }
    

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



