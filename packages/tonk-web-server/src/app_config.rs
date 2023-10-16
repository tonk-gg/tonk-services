use actix_web::web;
use crate::handlers::{action, game, player, building, vote, task};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(
        web::resource("/")
            .route(web::get().to(game::health_check))
    )
    .service(
        web::scope("/building")
            .service(
                web::resource("")
                    .route(web::post().to(building::post_building))
            )
    ).service(
        web::scope("/player")
            .service(
                web::scope("/{player_id}")
                    .service(
                        web::resource("")
                            .route(web::post().to(player::post_player))
                            .route(web::get().to(player::get_player))
                    )
            )
    ).service(
        web::scope("/game")
            .service(
                web::resource("")
                    .route(web::get().to(game::get_game))
                    .route(web::post().to(game::post_game))
            )
            .service(
                web::scope("/result")
                .service(
                    web::resource("")
                        .route(web::get().to(game::get_result))
                )
                .service(
                    web::scope("/{round_number}")
                        .service(
                            web::resource("")
                                .route(web::get().to(game::get_round_result))
                        )
                )
            )
            .service(
                web::scope("/{game_id}")
                    .service(
                        web::resource("/player")
                            .route(web::get().to(game::get_game_players))
                            .route(web::post().to(game::post_player))
                    )
            )
    ).service(
        web::scope("/action")
            .service(
                web::resource("")
                    .route(web::post().to(action::post_action))
            )
    ).service(
        web::scope("/task")
            .service(
                web::resource("")
                    .route(web::post().to(task::post_task))
                    .route(web::get().to(task::get_task))
            )
    ).service(
        web::resource("/vote")
            .route(web::post().to(vote::post_vote))
    );
}