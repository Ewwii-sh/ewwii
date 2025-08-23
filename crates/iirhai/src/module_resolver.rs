use rhai::{Engine, EvalAltResult, Module, ModuleResolver, Position, Scope, AST};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub struct SimpleFileResolver;

impl ModuleResolver for SimpleFileResolver {
    fn resolve(
        &self,
        engine: &Engine,
        source_path: Option<&str>,
        path: &str,
        _pos: Position,
    ) -> Result<Rc<Module>, Box<EvalAltResult>> {
        let mut file_path = PathBuf::from(path);

        if file_path.extension().is_none() {
            file_path.set_extension("rhai");
        }

        let base_dir = if let Some(src) = source_path {
            PathBuf::from(src).parent().map(|p| p.to_path_buf()).unwrap_or(
                std::env::current_dir().map_err(|e| {
                    EvalAltResult::ErrorSystem("getting current_dir".into(), e.into())
                })?,
            )
        } else {
            std::env::current_dir()
                .map_err(|e| EvalAltResult::ErrorSystem("getting current_dir".into(), e.into()))?
        };

        if !file_path.is_absolute() {
            file_path = base_dir.join(file_path);
        }

        let full_path = file_path
            .canonicalize()
            .map_err(|e| EvalAltResult::ErrorSystem(format!("resolving path: {path}"), e.into()))?;

        let script = fs::read_to_string(&full_path).map_err(|e| {
            EvalAltResult::ErrorSystem(format!("reading file: {full_path:?}"), e.into())
        })?;

        let ast: AST = engine.compile(&script)?;
        let scope = Scope::new();
        let mut module = Module::eval_ast_as_new(scope, &ast, engine)?;

        module.build_index();
        Ok(Rc::new(module))
    }
}

pub struct ChainedResolver<Res1, Res2> {
    pub first: Res1,
    pub second: Res2,
}

impl<R1: ModuleResolver, R2: ModuleResolver> ModuleResolver for ChainedResolver<R1, R2> {
    fn resolve(
        &self,
        engine: &Engine,
        source_path: Option<&str>,
        path: &str,
        pos: Position,
    ) -> Result<Rc<Module>, Box<EvalAltResult>> {
        self.first
            .resolve(engine, source_path, path, pos)
            .or_else(|_| self.second.resolve(engine, source_path, path, pos))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::ParseConfig;

    #[test]
    fn test_mod_resolver() -> Result<(), Box<dyn std::error::Error>> {
        let mut engine = Engine::new();

        // Set the custom module resolver into the 'Engine'.
        engine.set_module_resolver(SimpleFileResolver);

        let test_code = r#"
            import "/home/byson94/.config/ewwii/foo/baz" as baz;  // this 'import' statement will call
            baz::greet();
        "#;

        let mut parser = ParseConfig::new();
        println!("{:#?}", parser.eval_code(test_code));

        Ok(())
    }
}
