# include-lua #
include-lua is a crate that allows the embedding of a lua source tree into a Rust application binary. This tree can then be loaded into an [`rlua`](https://github.com/kyren/rlua) context, and code imported from it via `require`. 

## Basic Usage ##
First, create an instance of the `LuaModules` struct via the macro `include_lua!`. This macro takes a string literal parameter specifying a directory, relative to your crate's `src` folder. All `.lua` files in this directory and its subdirectories will be included as loadable modules. 

Once you've created a `LuaModules` struct, you can import it into an `rlua::Context` by calling `ctx.add_modules(modules)`. This is an extension method provided by a trait, so make sure you have a `use include_lua::*;` statement in your code. Once it has been called, any calls to `require` executed in that context will be able to load modules from the embedded source tree. 

## Advanced Usage ##
It is possible to specify a name to use for the `LuaModules` struct, which will appear in stacktraces from any code within it. Simply invoke the `include_lua` macro like `include_lua!("name": "path")`, instead of just `include_lua!("path")`.

If you would like to load modules in a custom environment for some reason, instead of `ctx.add_modules`, you can call `ctx.add_modules_with_env(modules, env)`, where `env` is a table that will be used as the `_ENV` value of all modules within the source tree.

The methods `ctx.make_searcher(modules)` and `ctx.make_searcher_with_env(modules, env)` are also available. They produce a piece of userdata that in Lua code, acts like a function to load a module from the source tree by name.

As they are lower-level methods, values returned by `make_searcher` or `make_searcher_with_env` do not cache modules like `require`. This means that if you want to avoid multiple calls with the same name loading multiple copies of the same module you will have to implement a wrapper in your lua code. 

## Example ##
See [example/main.rs](https://github.com/AlphaModder/include-lua/blob/master/example/src/main.rs) for a working example of the macro's use.

## Caveats ##
Currently, this crate does not support paths that contain non-unicode characters. Any files along these paths will be omitted from an `include_lua!` call.