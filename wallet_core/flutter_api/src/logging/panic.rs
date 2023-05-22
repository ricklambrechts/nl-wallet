use std::{backtrace::Backtrace, panic};

use tracing::error;

pub fn init_panic_logger() {
    panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::force_capture();

        // The payload may either be a reference to a [`String`] or a `&'static str`.
        let payload = panic_info.payload();
        let message = match (payload.downcast_ref::<String>(), payload.downcast_ref::<&'static str>()) {
            (Some(s), _) => Some(s.as_ref()),
            (_, Some(s)) => Some(*s),
            (_, _) => None,
        };

        // Log the panic message and backtrace, each on separate lines
        // because OSLog on iOS has a 1024 character limit.
        // See: https://stackoverflow.com/questions/39584707/nslog-on-devices-in-ios-10-xcode-8-seems-to-truncate-why/40283623#40283623
        //
        // Note that we need to use string formatting to prevent
        // the [`error!`] macro from printing the variable name.
        error!("Panic occurred: {}", message.unwrap_or("UNKNOWN"));
        backtrace
            .to_string()
            .split('\n')
            .filter(|backtrace_line| !backtrace_line.is_empty())
            .for_each(|backtrace_line| error!("{}", backtrace_line));
    }));
}
