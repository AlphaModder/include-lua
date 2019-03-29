use include_lua::*;
use rlua::Lua;

fn main() -> rlua::Result<()> {
    let lua = Lua::new();
    let modules = include_lua!("lib");
    lua.context(|ctx| -> rlua::Result<()> {
        ctx.add_modules(modules)?;
        println!("{}", ctx.load("require('alpha')").eval::<String>()?);
        println!("{}", ctx.load("require('test')").eval::<String>()?);
        Ok(())
    })
    
}
