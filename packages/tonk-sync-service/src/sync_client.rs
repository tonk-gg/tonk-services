use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use log::*;
use num_bigint::BigInt;
use redis::{Commands, ToRedisArgs};
use reqwest;
use tonk_shared_lib::{deserialize_struct, serialize_struct, Building, Location, Player};

#[derive(GraphQLQuery, Debug)]
#[graphql(schema_path = "schema.json", query_path = "src/dsplayers.graphql")]
struct DSPlayers;

#[derive(GraphQLQuery, Debug)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/dsbuilding_tiles.graphql"
)]
struct DSBuildingTiles;

pub struct SyncClient {
    client: reqwest::Client,
}

impl SyncClient {
    pub fn new() -> Self {
        let sync_client = Self {
            client: reqwest::Client::new(),
        };
        sync_client
    }
    pub async fn get_players(
        &self,
        con: &mut redis::Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let v = ds_players::Variables {
            game_id: "DOWNSTREAM".to_string(),
        };
        let res =
            post_graphql::<DSPlayers, _>(&self.client, "http://localhost:8080/query", v).await?;

        for entry in &res.data.unwrap().game.state.nodes {
            if let (Some(player), Some(location)) = (&entry.player, &entry.location) {
                // Do something with `player` and `location`

                let id = player.id.as_str();
                let location_coords = Location(
                    location.tile.coords[0].to_string(),
                    location.tile.coords[1].to_string(),
                    location.tile.coords[2].to_string(),
                    location.tile.coords[3].to_string(),
                );
                info!("{}", id);
                info!(
                    "{} {} {} {}",
                    location.tile.coords[0],
                    location.tile.coords[1],
                    location.tile.coords[2],
                    location.tile.coords[3]
                );
                let pstruct = Player {
                    id: id.to_string(),
                    location: location_coords,
                };

                let serialized_player = serialize_struct(pstruct);
                let key = format!("player:{}", id);

                let _: () = con.set(key, serialized_player)?;
            }
        }
        Ok(())
    }

    // What to do for the buildings?
    // Presumably, these aren't going to move
    // That means we can just register them once upfront

    // pub async fn get_buildings(&self, con: &mut redis::Connection) -> Result<(), Box<dyn std::error::Error>> {
    //     // let client = reqwest::Client::new();
    //     let v = ds_building_tiles::Variables {
    //         game_id: "DOWNSTREAM".to_string()
    //     };
    //     let res =
    //     post_graphql::<DSBuildingTiles, _>(&self.client, "http://localhost:8080/query", v).await?;

    //     for entry in &res.data.unwrap().game.state.nodes {
    //         if let Some(building) = &entry.building {
    //             // do some check here to make sure that it's a Depot building
    //             // we'll need to know the ids somehow beforehand I suppose
    //             // I suppose this will have to be set by some kind of admin?
    //             let depot_id = BigInt::from(con.get("depot").unwrap_or("0"));
    //             let tower_id = BigInt::from(con.get("tower").unwrap_or("0"));

    //             if let Some(kind) = building {
    //                 if kind.id == depot_id || {
    //                     let location_coords = Location(
    //                         entry.coords[0].to_string(),
    //                         entry.coords[1].to_string(),
    //                         entry.coords[2].to_string(),
    //                         entry.coords[3].to_string()
    //                     );

    //                 }
    //             }

    //         }
    //     }
    //     // info!("{}", res.data.unwrap().game.id);
    //     Ok(())
    // }
}
