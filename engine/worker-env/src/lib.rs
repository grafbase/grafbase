use worker::Var;

pub enum VarType {
    Var,
    Secret,
}

pub trait EnvExt {
    fn var_get_list_opt<T>(&self, var_type: VarType, var_name: &str, delimiter: char) -> worker::Result<Option<Vec<T>>>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display;
    fn var_get_list<T>(&self, var_type: VarType, var_name: &str, delimiter: char) -> worker::Result<Vec<T>>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display;
    fn var_get_opt<T>(&self, var_type: VarType, var_name: &str) -> worker::Result<Option<T>>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display;
    fn var_get<T>(&self, var_type: VarType, var_name: &str) -> worker::Result<T>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display;
    /// To fetch the variable directly without loading it into WASM memory
    fn js_var_get(&self, var_type: VarType, var_name: &str) -> worker::Result<Var>;
}

impl EnvExt for worker::Env {
    fn var_get_list_opt<T>(&self, var_type: VarType, var_name: &str, delimiter: char) -> worker::Result<Option<Vec<T>>>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        let value = match var_type {
            VarType::Var => self.var(var_name),
            VarType::Secret => self.secret(var_name),
        };
        value
            .ok()
            .map(|string| {
                string
                    .to_string()
                    .split(delimiter)
                    .map(|string| {
                        string.parse().map_err(|err| {
                            worker::Error::RustError(format!(
                                "could not parse \"{string}\" as a `{type_name}`: `{err}`",
                                type_name = std::any::type_name::<T>()
                            ))
                        })
                    })
                    .collect::<Result<_, _>>()
            })
            .transpose()
    }
    fn var_get_list<T>(&self, var_type: VarType, var_name: &str, delimiter: char) -> worker::Result<Vec<T>>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        self.var_get_list_opt(var_type, var_name, delimiter)?
            .ok_or_else(|| worker::Error::BindingError(var_name.to_string()))
    }
    fn var_get_opt<T>(&self, var_type: VarType, var_name: &str) -> worker::Result<Option<T>>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        Ok(self
            .var_get_list_opt(var_type, var_name, '\0')?
            .and_then(|mut list| list.pop()))
    }
    fn var_get<T>(&self, var_type: VarType, var_name: &str) -> worker::Result<T>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        Ok(self.var_get_list(var_type, var_name, '\0')?.pop().unwrap())
    }
    fn js_var_get(&self, var_type: VarType, var_name: &str) -> worker::Result<Var> {
        match var_type {
            VarType::Var => self.var(var_name),
            VarType::Secret => self.secret(var_name),
        }
    }
}
