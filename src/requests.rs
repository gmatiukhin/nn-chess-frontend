use tokio::sync::mpsc::{self, Sender};

use anyhow::Result;
use poll_promise::Promise;
use shakmaty::fen::Fen;
use web_types::*;

#[derive(Debug)]
pub enum RequestLoopComm {
    FetchEngines(oneshot::Sender<Result<EngineDirectory>>),
    FetchEngineDescription(EngineRef, oneshot::Sender<Result<EngineDescription>>),
    FetchPosEval(
        EngineVariant,
        Fen,
        oneshot::Sender<Result<GameMoveResponse>>,
    ),
}

pub fn run_request_loop() -> mpsc::UnboundedSender<RequestLoopComm> {
    let (request_sender, mut request_receiver) = mpsc::unbounded_channel::<RequestLoopComm>();
    let _ = Promise::spawn_local(async move {
        while let Some(comm) = request_receiver.recv().await {
            log::debug!("Received request: {comm:?}");
            match comm {
                RequestLoopComm::FetchEngines(response_sender) => {
                    let resp = get_engines().await;
                    log::info!("Received engine directory result: {resp:?}");
                    let _ = response_sender.send(resp);
                }
                RequestLoopComm::FetchEngineDescription(engine_ref, response_sender) => {
                    let resp = get_engine_description(engine_ref.clone()).await;
                    log::info!("Received engine description result: {resp:?}");
                    let _ = response_sender.send(resp);
                }
                RequestLoopComm::FetchPosEval(engine_variant, fen, response_sender) => {
                    let resp = get_position_evaluation(engine_variant.clone(), fen.clone()).await;
                    log::info!("Received game move result: {resp:?}");
                    let _ = response_sender.send(resp);
                }
            }
        }
    });
    request_sender
}

async fn get_engines() -> Result<EngineDirectory> {
    Ok(reqwest::get("https://api.unchessful.games/")
        .await?
        .json()
        .await?)
}

async fn get_engine_description(engine_ref: EngineRef) -> Result<EngineDescription> {
    Ok(reqwest::get(engine_ref.entrypoint_url)
        .await?
        .json()
        .await?)
}

async fn get_position_evaluation(
    engine_varian: EngineVariant,
    fen: Fen,
) -> Result<GameMoveResponse> {
    let client = reqwest::Client::new();
    let data = GameMoveRequest {
        fen: fen.to_string(),
    };
    Ok(client
        .post(engine_varian.game_url)
        .json(&data)
        .send()
        .await?
        .json()
        .await?)
}
