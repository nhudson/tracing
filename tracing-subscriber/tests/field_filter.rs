#![cfg(feature = "env-filter")]

use tracing::{self, collect::with_default, Level};
use tracing_mock::*;
use tracing_subscriber::{filter::EnvFilter, prelude::*};

#[test]
#[cfg_attr(not(flaky_tests), ignore)]
fn field_filter_events() {
    let filter: EnvFilter = "[{thing}]=debug".parse().expect("filter should parse");
    let (subscriber, finished) = collector::mock()
        .event(
            event::expect()
                .at_level(Level::INFO)
                .with_fields(field::expect("thing")),
        )
        .event(
            event::expect()
                .at_level(Level::DEBUG)
                .with_fields(field::expect("thing")),
        )
        .only()
        .run_with_handle();
    let subscriber = subscriber.with(filter);

    with_default(subscriber, || {
        tracing::trace!(disabled = true);
        tracing::info!("also disabled");
        tracing::info!(thing = 1);
        tracing::debug!(thing = 2);
        tracing::trace!(thing = 3);
    });

    finished.assert_finished();
}

#[test]
#[cfg_attr(not(flaky_tests), ignore)]
fn field_filter_spans() {
    let filter: EnvFilter = "[{enabled=true}]=debug"
        .parse()
        .expect("filter should parse");
    let (subscriber, finished) = collector::mock()
        .enter(span::expect().named("span1"))
        .event(
            event::expect()
                .at_level(Level::INFO)
                .with_fields(field::expect("something")),
        )
        .exit(span::expect().named("span1"))
        .enter(span::expect().named("span2"))
        .exit(span::expect().named("span2"))
        .enter(span::expect().named("span3"))
        .event(
            event::expect()
                .at_level(Level::DEBUG)
                .with_fields(field::expect("something")),
        )
        .exit(span::expect().named("span3"))
        .only()
        .run_with_handle();
    let subscriber = subscriber.with(filter);

    with_default(subscriber, || {
        tracing::trace!("disabled");
        tracing::info!("also disabled");
        tracing::info_span!("span1", enabled = true).in_scope(|| {
            tracing::info!(something = 1);
        });
        tracing::debug_span!("span2", enabled = false, foo = "hi").in_scope(|| {
            tracing::warn!(something = 2);
        });
        tracing::trace_span!("span3", enabled = true, answer = 42).in_scope(|| {
            tracing::debug!(something = 2);
        });
    });

    finished.assert_finished();
}

#[test]
fn record_after_created() {
    let filter: EnvFilter = "[{enabled=true}]=debug"
        .parse()
        .expect("filter should parse");
    let (subscriber, finished) = collector::mock()
        .enter(span::expect().named("span"))
        .exit(span::expect().named("span"))
        .record(
            span::expect().named("span"),
            field::expect("enabled").with_value(&true),
        )
        .enter(span::expect().named("span"))
        .event(event::expect().at_level(Level::DEBUG))
        .exit(span::expect().named("span"))
        .only()
        .run_with_handle();
    let subscriber = subscriber.with(filter);

    with_default(subscriber, || {
        let span = tracing::info_span!("span", enabled = false);
        span.in_scope(|| {
            tracing::debug!("i'm disabled!");
        });

        span.record("enabled", true);
        span.in_scope(|| {
            tracing::debug!("i'm enabled!");
        });

        tracing::debug!("i'm also disabled");
    });

    finished.assert_finished();
}
