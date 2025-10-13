use crate::error::{format_eval_error, format_parse_error};
use crate::parser::ParseConfig;
use crate::updates::ReactiveVarStore;
use rhai::{Dynamic, Engine, EvalAltResult, Module, ModuleResolver, Position, AST};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub struct SimpleFileResolver {
    pub pl_handler_store: Option<ReactiveVarStore>,
}

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

        let ast: AST = engine.compile(&script).map_err(|e| {
            Box::new(EvalAltResult::ErrorSystem(
                "module_parse_failed".into(),
                format_parse_error(&e, &script, full_path.to_str()).into(),
            ))
        })?;

        let mut scope = ParseConfig::initial_poll_listen_scope(&script).map_err(|e| {
            EvalAltResult::ErrorSystem(
                format!("error setting up default variables: {full_path:?}"),
                e.into(),
            )
        })?;

        match &self.pl_handler_store {
            Some(val) => {
                let name_to_val: &HashMap<String, String> = &*val.read().unwrap();

                for (name, val) in name_to_val {
                    scope.set_value(name.clone(), Dynamic::from(val.clone()));
                }
            }
            None => {}
        }

        let mut module = Module::eval_ast_as_new(scope, &ast, engine).map_err(|e| {
            Box::new(EvalAltResult::ErrorSystem(
                "module_eval_failed".into(),
                format_eval_error(&e, &script, engine, full_path.to_str()).into(),
            ))
        })?;

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
        match self.first.resolve(engine, source_path, path, pos) {
            Ok(m) => Ok(m),
            Err(e1) => {
                if let EvalAltResult::ErrorSystem(msg, _) = e1.as_ref() {
                    if msg == "module_eval_failed" || msg == "module_parse_failed" {
                        return Err(e1);
                    }
                }

                log::trace!(
                    "Error executing resolver 1, falling back to resolver 2. Error details: {}",
                    e1
                );
                match self.second.resolve(engine, source_path, path, pos) {
                    Ok(m) => Ok(m),
                    Err(e2) => Err(Box::new(EvalAltResult::ErrorSystem(
                        format!(
                            "Both resolvers failed; first: {}, second (possibly unrelated): {}",
                            e1, e2
                        ),
                        Box::new(e2),
                    ))),
                }
            }
        }
    }
}
