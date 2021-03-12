use slack_morphism::prelude::*;
use slack_morphism_hyper::*;

use futures::stream::BoxStream;
use futures::TryStreamExt;
use std::time::Duration;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use log::*;

use std::sync::Arc;

mod templates;
use templates::*;

async fn send_message(
    channel: &SlackChannelId,
    content: SlackMessageContent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("triggered");
    let request = SlackApiChatPostMessageRequest::new(channel.clone(), content);

    let hyper_connector = SlackClientHyperConnector::new();
    let client = SlackClient::new(hyper_connector);
    let token_value: SlackApiTokenValue = config_env_var("SLACK_BOT_TOKEN")?.into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let session = client.open_session(&token);

    let response = session.chat_post_message(&request).await?;
    println!("chat_post_message response: {:#?}", response);
    Ok(())
}

async fn test_push_events_function(event: SlackPushEvent, _client: Arc<SlackHyperClient>) {
    println!("{:#?}", event);
    if let SlackPushEvent::EventCallback(callback) = event {
        if let SlackEventCallbackBody::Message(message) = callback.event {
            if let Some(channel_type) = message.origin.channel_type {
                if channel_type == SlackChannelType("im".to_owned())
                    && message.sender.bot_id.is_none()
                {
                    println!("about to await");
                    let resp = send_message(
                        &message
                            .origin
                            .channel
                            .expect("Message didn't have a channel"),
                        // Can't just re-use message.content: https://github.com/abdolence/slack-morphism-rust/issues/24
                        SlackMessageContent {
                            text: message.content.text.clone(),
                            blocks: None,
                        },
                    )
                    .await; // TODO catch failures here and log them
                    println!("send_message finished: {:#?}", resp);
                }
            }
        }
    }
}

fn test_error_handler(
    err: Box<dyn std::error::Error + Send + Sync>,
    _client: Arc<SlackHyperClient>,
) {
    println!("{:#?}", err);
}

async fn test_server(
    client: Arc<SlackHyperClient>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 80));
    info!("Loading server: {}", addr);

    async fn your_others_routes(
        _req: Request<Body>,
    ) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
        Response::builder()
            .body("Hey, this is a default users route handler".into())
            .map_err(|e| e.into())
    }

    let push_events_config = Arc::new(SlackPushEventsListenerConfig::new(config_env_var(
        "SLACK_SIGNING_SECRET",
    )?));

    let make_svc = make_service_fn(move |_| {
        let thread_push_events_config = push_events_config.clone();
        let listener_environment = SlackClientEventsListenerEnvironment::new(client.clone())
            .with_error_handler(test_error_handler);
        let listener = SlackClientEventsHyperListener::new(listener_environment);
        async move {
            let routes = chain_service_routes_fn(
                listener
                    .push_events_service_fn(thread_push_events_config, test_push_events_function),
                your_others_routes,
            );

            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(service_fn(routes))
        }
    });

    let server = hyper::server::Server::bind(&addr).serve(make_svc);
    server.await.map_err(|e| {
        error!("Server error: {}", e);
        e.into()
    })
}

fn init_log() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use fern::colors::{Color, ColoredLevelConfig};

    let colors_level = ColoredLevelConfig::new()
        .info(Color::Green)
        .warn(Color::Magenta);

    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}{}\x1B[0m",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors_level.color(record.level()),
                format_args!(
                    "\x1B[{}m",
                    colors_level.get_color(&record.level()).to_fg_str()
                ),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        // - and per-module overrides
        .level_for("hyper", log::LevelFilter::Info)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        // Apply globally
        .apply()?;

    Ok(())
}

pub fn config_env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_log()?;
    let hyper_connector = SlackClientHyperConnector::new();
    let client: Arc<SlackHyperClient> = Arc::new(SlackClient::new(hyper_connector));
    test_server(client.clone()).await?;

    Ok(())
}
