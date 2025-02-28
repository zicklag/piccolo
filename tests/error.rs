mod sizes;

use piccolo::{error::LuaError, AnyCallback, Closure, Error, Lua, StaticError, Thread, Value};
use thiserror::Error;

#[test]
fn error_unwind() -> Result<(), StaticError> {
    let mut lua = Lua::core();

    let thread = lua.try_run(|ctx| {
        let closure = Closure::load(
            ctx,
            &br#"
                function do_error()
                    error('test error')
                end

                do_error()
            "#[..],
        )?;
        let thread = Thread::new(&ctx);
        thread.start(ctx, closure.into(), ())?;
        Ok(ctx.state.registry.stash(&ctx, thread))
    })?;

    lua.finish_thread(&thread);
    lua.try_run(|ctx| {
        match ctx.state.registry.fetch(&thread).take_return::<()>(ctx)? {
            Err(Error::Lua(LuaError(Value::String(s)))) => assert!(s == "test error"),
            _ => panic!(),
        }
        Ok(())
    })
}

#[test]
fn error_tostring() -> Result<(), StaticError> {
    let mut lua = Lua::core();

    #[derive(Debug, Error)]
    #[error("test error")]
    struct TestError;

    let thread = lua.try_run(|ctx| {
        let callback = AnyCallback::from_fn(&ctx, |_, _, _| Err(TestError.into()));
        ctx.state.globals.set(ctx, "callback", callback)?;

        let closure = Closure::load(
            ctx,
            &br#"
                local r, e = pcall(callback)
                assert(not r)
                assert(tostring(e) == "test error")
            "#[..],
        )?;

        let thread = Thread::new(&ctx);
        thread.start(ctx, closure.into(), ())?;
        Ok(ctx.state.registry.stash(&ctx, thread))
    })?;

    lua.run_thread(&thread)
}
