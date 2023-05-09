use anyhow::Result;
use once_cell::sync::OnceCell;
use tokio::runtime::{Builder, Runtime};

static ASYNC_RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn init_async_runtime() -> Result<()> {
    _ = ASYNC_RUNTIME.get_or_try_init(|| Builder::new_multi_thread().enable_all().build())?;

    Ok(())
}

pub fn get_async_runtime() -> &'static Runtime {
    ASYNC_RUNTIME
        .get()
        .expect("Wallet must be initialized. Please execute `init()` first.")
}

#[cfg(test)]
mod tests {
    async fn plus(left: i32, right: i32) -> i32 {
        left + right
    }

    #[macros::async_runtime]
    async fn add(left: i32, right: i32) -> i32 {
        plus(left, right).await
    }

    #[test]
    fn can_invoke_async_function_in_core() {
        let _ = crate::async_runtime::init_async_runtime();
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
