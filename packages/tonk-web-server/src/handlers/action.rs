use actix_web::{web, Error, HttpResponse, HttpRequest};
use tonk_shared_lib::{Game, Player, Action, GameStatus, Task, Role};
use tonk_shared_lib::redis_helper::*;
use serde::{Deserialize, Serialize};
use log::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionQuery {
    player_id: String,
    secret_key: String 
}

// USED TO POISON OTHER PLAYERS DURING THE TASK ROUND
pub async fn post_action(_id: web::Json<Action>, _query: web::Query<ActionQuery>, req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut redis = RedisHelper::init().await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let action = _id.0;
    let game: Game = redis.get_key("game").await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;
    let round = game.time.unwrap().round;
    if round != action.round {
        return Err(actix_web::error::ErrorBadRequest("Improper round in request"));
    }
    if game.status != GameStatus::Tasks {
        return Err(actix_web::error::ErrorForbidden("The game is not in the task round"));
    }
    
    let player_id = &_query.player_id;
    let player_key = format!("player:{}", player_id);
    let player: Player = redis.get_key(&player_key).await.map_err(|e| {
        error!("{:?}", e);
        actix_web::error::ErrorInternalServerError("Unknown error")
    })?;

    if *player.role.as_ref().unwrap() != Role::Bugged {
        return Err(actix_web::error::ErrorForbidden("You cannot take this action"));
    }

    let nearby_players = player.nearby_players.unwrap();

    let target_is_near = nearby_players.iter().find(|e| {
        e.id == action.poison_target.id
    });
    if target_is_near.is_none() {
        return Err(actix_web::error::ErrorForbidden("The target is not within range"));
    }

    let action_key = format!("action:{}:{}:{}", game.id, round, player.id);
    let exists: Result<Action, _> = redis.get_key(&action_key).await;
    if exists.is_err() {
        let mut updated_action = action.clone();
        let interrupted_task_key = format!("task:{}:{}:{}", game.id, round, action.poison_target.id);
        let task_result: Task = redis.get_key(&interrupted_task_key).await.map_err(|e| {
            error!("{:?}", e);
            actix_web::error::ErrorInternalServerError("Unknown error")
        })?;
        if !task_result.complete {
            updated_action.interrupted_task = true;
        }

        let _ = redis.set_key(&action_key, &updated_action).await.map_err(|e| {
            error!("{:?}", e);
            actix_web::error::ErrorInternalServerError("Unknown error")
        })?;
        redis.add_to_index("game:actions", &action_key).await.map_err(|e| {
            error!("{:?}", e);
            actix_web::error::ErrorInternalServerError("Unknown error")
        })?;
    } else {
        return Err(actix_web::error::ErrorForbidden("You have already taken an action this round"));
    }

    Ok(HttpResponse::Ok().finish())
}