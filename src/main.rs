// SPDX-License-Identifier: GPL-3.0-only

mod applet;
mod config;
mod i18n;
mod weather;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    let _ = tracing_log::LogTracer::init();

    tracing::info!("Starting tempest applet v{}", VERSION);

    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);

    cosmic::applet::run::<applet::Tempest>(())
}
