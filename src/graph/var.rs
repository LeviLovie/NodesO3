use serde::{Deserialize, Serialize};

pub type CustomType = (String, String); // (type_name, value)

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Type {
    Bool,
    Int,
    Float,
    String,
    Custom(String),
    Multi(Vec<Type>),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Bool => write!(f, "Bool"),
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::String => write!(f, "String"),
            Type::Custom(s) => write!(f, "Custom({})", s),
            Type::Multi(types) => {
                let types_str: Vec<String> = types.iter().map(|t| t.to_string()).collect();
                write!(f, "[{}]", types_str.join(", "))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Var {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Custom(CustomType),
}

impl Var {
    pub fn type_(&self) -> Type {
        match self {
            Var::Bool(_) => Type::Bool,
            Var::Int(_) => Type::Int,
            Var::Float(_) => Type::Float,
            Var::String(_) => Type::String,
            Var::Custom((name, _)) => Type::Custom(name.clone()),
        }
    }
}

impl std::fmt::Debug for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Var::Bool(b) => write!(f, "{}({})", self.type_(), b),
            Var::Int(i) => write!(f, "{}({})", self.type_(), i),
            Var::Float(fl) => write!(f, "{}({})", self.type_(), fl),
            Var::String(s) => write!(f, "{}(\"{}\")", self.type_(), s),
            Var::Custom((name, value)) => write!(f, "{}({})", name, value),
        }
    }
}

impl std::fmt::Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Var::Bool(b) => write!(f, "{}", b),
            Var::Int(i) => write!(f, "{}", i),
            Var::Float(fl) => write!(f, "{}", fl),
            Var::String(s) => write!(f, "\"{}\"", s),
            Var::Custom(s) => write!(f, "({}: {})", s.1, s.0),
        }
    }
}

impl From<&str> for Var {
    fn from(s: &str) -> Self {
        Var::String(s.to_string())
    }
}

impl From<String> for Var {
    fn from(s: String) -> Self {
        Var::String(s)
    }
}

impl TryInto<String> for Var {
    type Error = Self;
    fn try_into(self) -> Result<String, Self> {
        match self {
            Var::String(s) => Ok(s),
            _ => Err(self),
        }
    }
}

impl From<bool> for Var {
    fn from(b: bool) -> Self {
        Var::Bool(b)
    }
}

impl TryInto<bool> for Var {
    type Error = Self;
    fn try_into(self) -> Result<bool, Self> {
        match self {
            Var::Bool(b) => Ok(b),
            _ => Err(self),
        }
    }
}

impl From<i32> for Var {
    fn from(i: i32) -> Self {
        Var::Int(i as i64)
    }
}

impl TryInto<i32> for Var {
    type Error = Self;
    fn try_into(self) -> Result<i32, Self> {
        match self {
            Var::Int(i) => Ok(i as i32),
            _ => Err(self),
        }
    }
}

impl From<i64> for Var {
    fn from(i: i64) -> Self {
        Var::Int(i)
    }
}

impl TryInto<i64> for Var {
    type Error = Self;
    fn try_into(self) -> Result<i64, Self> {
        match self {
            Var::Int(i) => Ok(i),
            _ => Err(self),
        }
    }
}

impl From<u32> for Var {
    fn from(i: u32) -> Self {
        Var::Int(i as i64)
    }
}

impl TryInto<u32> for Var {
    type Error = Self;
    fn try_into(self) -> Result<u32, Self> {
        match self {
            Var::Int(i) => Ok(i as u32),
            _ => Err(self),
        }
    }
}

impl From<u64> for Var {
    fn from(i: u64) -> Self {
        Var::Int(i as i64)
    }
}

impl TryInto<u64> for Var {
    type Error = Self;
    fn try_into(self) -> Result<u64, Self> {
        match self {
            Var::Int(i) => Ok(i as u64),
            _ => Err(self),
        }
    }
}

impl From<f32> for Var {
    fn from(f: f32) -> Self {
        Var::Float(f as f64)
    }
}

impl TryInto<f32> for Var {
    type Error = Self;
    fn try_into(self) -> Result<f32, Self> {
        match self {
            Var::Float(f) => Ok(f as f32),
            _ => Err(self),
        }
    }
}

impl From<f64> for Var {
    fn from(f: f64) -> Self {
        Var::Float(f)
    }
}

impl TryInto<f64> for Var {
    type Error = Self;
    fn try_into(self) -> Result<f64, Self> {
        match self {
            Var::Float(f) => Ok(f),
            _ => Err(self),
        }
    }
}

impl From<CustomType> for Var {
    fn from(c: CustomType) -> Self {
        Var::Custom(c)
    }
}

impl From<(&str, &str)> for Var {
    fn from(c: (&str, &str)) -> Self {
        Var::from((c.0.to_string(), c.1.to_string()))
    }
}

impl PartialEq for Var {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Var::Bool(a), Var::Bool(b)) => a == b,
            (Var::Int(a), Var::Int(b)) => a == b,
            (Var::Float(a), Var::Float(b)) => a == b,
            (Var::String(a), Var::String(b)) => a == b,
            (Var::Custom((a_name, a_value)), Var::Custom((b_name, b_value))) => {
                a_name == b_name && a_value == b_value
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{CustomType, Type, Var};

    #[test]
    fn test_str() {
        let str = "Hello, World!";
        let var: Var = str.into();
        assert_eq!(var.to_string(), "\"Hello, World!\"");
        assert_eq!(var.type_().to_string(), "String");
        assert_eq!(format!("{:?}", var), "String(\"Hello, World!\")");
    }

    #[test]
    fn test_string() {
        let string = String::from("Hello, World!");
        let var_string: Var = string.into();
        assert_eq!(var_string.to_string(), "\"Hello, World!\"");
        assert_eq!(var_string, var_string);
        assert_eq!(var_string.type_().to_string(), "String");
    }

    #[test]
    fn test_string_eq() {
        assert_eq!(Var::from("Hello, World!"), Var::from("Hello, World!"));
        assert_ne!(Var::from("Hello, World!"), Var::from("Hello!"));
        assert_ne!(Var::from("Hello, World!"), Var::from(0));
    }

    #[test]
    fn test_bool() {
        let b = true;
        let var: Var = b.into();
        assert_eq!(var.to_string(), "true");
        assert_eq!(var.type_().to_string(), "Bool");
        assert_eq!(format!("{:?}", var), "Bool(true)");
    }

    #[test]
    fn test_bool_eq() {
        assert_eq!(Var::from(true), Var::from(true));
        assert_ne!(Var::from(true), Var::from(false));
        assert_ne!(Var::from(true), Var::from(0));
    }

    #[test]
    fn test_i32() {
        let i: i32 = 42;
        let var: Var = i.into();
        assert_eq!(Var::from(42i32).to_string(), "42");
        assert_eq!(var.type_().to_string(), "Int");
        assert_eq!(format!("{:?}", var), "Int(42)");
    }

    #[test]
    fn test_i32_eq() {
        assert_eq!(Var::from(42), Var::from(42));
        assert_ne!(Var::from(42), Var::from(43));
        assert_ne!(Var::from(42), Var::from(42.0));
        assert_ne!(Var::from(42), Var::from("42"));
    }

    #[test]
    fn test_i64() {
        let i: i64 = 42;
        let var: Var = i.into();
        assert_eq!(var.to_string(), "42");
        assert_eq!(var.type_().to_string(), "Int");
        assert_eq!(format!("{:?}", var), "Int(42)");
    }

    #[test]
    fn test_i64_eq() {
        assert_eq!(Var::from(42i64), Var::from(42i64));
        assert_ne!(Var::from(42i64), Var::from(43i64));
        assert_ne!(Var::from(42i64), Var::from(42.0));
        assert_ne!(Var::from(42i64), Var::from("42"));
    }

    #[test]
    fn test_u32() {
        let i: u32 = 42;
        let var: Var = i.into();
        assert_eq!(var.to_string(), "42");
        assert_eq!(var.type_().to_string(), "Int");
        assert_eq!(format!("{:?}", var), "Int(42)");
    }

    #[test]
    fn test_u32_eq() {
        assert_eq!(Var::from(42u32), Var::from(42u32));
        assert_ne!(Var::from(42u32), Var::from(43u32));
        assert_ne!(Var::from(42u32), Var::from(42.0));
        assert_ne!(Var::from(42u32), Var::from("42"));
    }

    #[test]
    fn test_u64() {
        let i: u64 = 42;
        let var: Var = i.into();
        assert_eq!(var.to_string(), "42");
        assert_eq!(var.type_().to_string(), "Int");
        assert_eq!(format!("{:?}", var), "Int(42)");
    }

    #[test]
    fn test_u64_eq() {
        assert_eq!(Var::from(42u64), Var::from(42u64));
        assert_ne!(Var::from(42u64), Var::from(43u64));
        assert_ne!(Var::from(42u64), Var::from(42.0));
        assert_ne!(Var::from(42u64), Var::from("42"));
    }

    #[test]
    fn test_f32() {
        let f: f32 = 2.0;
        let var: Var = f.into();
        assert_eq!(var.to_string(), "2");
        assert_eq!(var.type_().to_string(), "Float");
        assert_eq!(format!("{:?}", var), "Float(2)");
    }

    #[test]
    fn test_f32_eq() {
        assert_eq!(Var::from(3.1f32), Var::from(3.1f32));
        assert_ne!(Var::from(3.1f32), Var::from(3.2f32));
        assert_ne!(Var::from(3.1f32), Var::from(3.1));
        assert_ne!(Var::from(3.1f32), Var::from("3.1"));
    }

    #[test]
    fn test_f64() {
        let f: f64 = 3.1;
        let var: Var = f.into();
        assert_eq!(var.to_string(), "3.1");
        assert_eq!(var.type_().to_string(), "Float");
        assert_eq!(format!("{:?}", var), "Float(3.1)");
    }

    #[test]
    fn test_f64_eq() {
        assert_eq!(Var::from(3.1f64), Var::from(3.1f64));
        assert_ne!(Var::from(3.1f64), Var::from(3.2f64));
        assert_ne!(Var::from(3.1f64), Var::from("3.1"));
    }

    #[test]
    fn test_custom() {
        let custom: CustomType = ("Point".to_string(), "(1, 2)".to_string());
        let var: Var = custom.into();
        assert_eq!(var.to_string(), "((1, 2): Point)");
        assert_eq!(var.type_().to_string(), "Custom(Point)");
        assert_eq!(format!("{:?}", var), "Point((1, 2))");

        let custom = ("Point", "(1, 2)");
        let var: Var = custom.into();
        assert_eq!(var.to_string(), "((1, 2): Point)");
        assert_eq!(var.type_().to_string(), "Custom(Point)");
        assert_eq!(format!("{:?}", var), "Point((1, 2))");
    }

    #[test]
    fn test_custom_eq() {
        let custom1: CustomType = ("Point".to_string(), "(1, 2)".to_string());
        let custom2: CustomType = ("Point".to_string(), "(1, 2)".to_string());
        let custom3: CustomType = ("Point".to_string(), "(2, 3)".to_string());
        let custom4: CustomType = ("Vector".to_string(), "(1, 2)".to_string());
        assert_eq!(Var::from(custom1.clone()), Var::from(custom2));
        assert_ne!(Var::from(custom1.clone()), Var::from(custom3));
        assert_ne!(Var::from(custom1), Var::from(custom4));
        assert_ne!(Var::from(("Point", "(1, 2)")), Var::from(0));
    }

    #[test]
    fn test_multi() {
        let multi_type = Type::Multi(vec![
            Type::Int,
            Type::String,
            Type::Custom("Point".to_string()),
        ]);
        assert_eq!(multi_type.to_string(), "[Int, String, Custom(Point)]");
    }
}
