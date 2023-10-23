use tokio::sync::mpsc::{self, Sender};

use poll_promise::Promise;
use reqwest::Result;
use shakmaty::fen::Fen;
use web_types::*;

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
    let (request_sender, request_receiver) = mpsc::unbounded_channel::<RequestLoopComm>();
    let _ = Promise::spawn_local(async move {
        for comm in request_receiver.recv().await.iter() {
            match comm {
                RequestLoopComm::FetchEngines(response_sender) => {
                    let resp = get_engines().await;
                    let _ = response_sender.send(resp);
                }
                RequestLoopComm::FetchEngineDescription(engine_ref, response_sender) => {
                    let resp = get_engine_description(engine_ref.clone()).await;
                    let _ = response_sender.send(resp);
                }
                RequestLoopComm::FetchPosEval(engine_variant, fen, response_sender) => {
                    let resp = get_position_evaluation(engine_variant.clone(), fen.clone()).await;
                    let _ = response_sender.send(resp);
                }
            }
        }
    });
    request_sender
}

async fn get_engines() -> Result<EngineDirectory> {
    reqwest::get("https://api.unchessful.games/")
        .await?
        .json()
        .await
}

async fn get_engine_description(engine_ref: EngineRef) -> Result<EngineDescription> {
    reqwest::get(engine_ref.entrypoint_url).await?.json().await
}

async fn get_position_evaluation(
    engine_varian: EngineVariant,
    fen: Fen,
) -> Result<GameMoveResponse> {
    let client = reqwest::Client::new();
    let data = GameMoveRequest {
        fen: fen.to_string(),
    };
    client
        .post(engine_varian.game_url)
        .json(&data)
        .send()
        .await?
        .json()
        .await
}
