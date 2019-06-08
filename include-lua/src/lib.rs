use std::collections::HashMap;

use proc_macro_hack::proc_macro_hack;
use rlua::{Result, Context, UserData, UserDataMethods, MetaMethod, Value, Table, RegistryKey};

/// A macro that embeds a lua source tree on disk into the binary, similarly to how `include_str!`
/// can include a single file. Called like `include_lua!("name": "path")`, where name is a label
/// that appears in lua stacktraces involving code loaded from the tree, and path specifies a folder
/// relative to `src/` in which the tree can be found. `name` defaults to `path` if omitted.
#[proc_macro_hack]
pub use include_lua_macro::include_lua;

/// Represents a Lua source tree embedded into a binary via [`include_lua!`][include_lua].
pub struct LuaModules {
    files: HashMap<String, (String, String)>,
    prefix: String,
}

impl LuaModules {
    #[doc(hidden)] // This is not a public API!
    pub fn __new(files: HashMap<String, (String, String)>, prefix: &str) -> LuaModules {
        LuaModules { files: files, prefix: prefix.to_string() }
    }
}

/// A piece of [`UserData`][UserData] that acts like a Lua searcher.
/// When called as a function with a single string parameter, attempts to load
/// (but not execute) a module by that name. If no module is found, returns nil.
pub struct Searcher(LuaModules, RegistryKey);

impl UserData for Searcher {
     fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Call, |ctx, this, value: String| {
            Ok(match this.0.files.get(&value) {
                Some((source, path)) => {
                    Value::Function(ctx.load(source)
                        .set_name(&format!("{}\\{} (virtual)", &this.0.prefix, path))?
                        .set_environment(ctx.registry_value::<Table>(&this.1)?)?
                        .into_function()?
                    )
                }
                None => Value::Nil,
            })
        });
    }
}

/// An extension trait for [`Context`][Context] that allows the loading of [`LuaModules`][LuaModules] instances.
pub trait ContextExt<'a> {
    /// Makes the source tree represented by `modules` accessible to `require` calls within this context.
    fn add_modules(&self, modules: LuaModules) -> Result<()>;

    /// Makes the source tree represented by `modules` accessible to `require` calls within this context.
    /// All modules loaded from the source tree will have their environment set to `environment`.
    fn add_modules_with_env(&self, modules: LuaModules, environment: Table<'a>) -> Result<()>;

    /// Creates a [`Searcher`][Searcher] instance from the given [`LuaModules`][LuaModules] instance.
    fn make_searcher(&self, modules: LuaModules) -> Result<Searcher>;

    /// Creates a [`Searcher`][Searcher] instance from the given [`LuaModules`][LuaModules] instance.
    /// All modules loaded by the searcher will have their environment set to `environment`.
    fn make_searcher_with_env(&self, modules: LuaModules, environment: Table<'a>) -> Result<Searcher>;
}

impl<'a> ContextExt<'a> for Context<'a> {
    fn add_modules(&self, modules: LuaModules) -> Result<()> {
        self.add_modules_with_env(modules, self.globals())
    }

    fn add_modules_with_env(&self, modules: LuaModules, environment: Table<'a>) -> Result<()> {
        let searchers: Table = self.globals().get::<_, Table>("package")?.get("searchers")?;
        searchers.set(searchers.len()? + 1, self.make_searcher_with_env(modules, environment)?)
    }

    fn make_searcher(&self, modules: LuaModules) -> Result<Searcher> {
        self.make_searcher_with_env(modules, self.globals())
    }

    fn make_searcher_with_env(&self, modules: LuaModules, environment: Table<'a>) -> Result<Searcher> {
        Ok(Searcher(modules, self.create_registry_value(environment)?))
    }
}