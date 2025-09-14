// SPDX-FileCopyrightText: 2025 FreshlyBakedCake
//
// SPDX-License-Identifier: MIT

use clap::Parser as _;

mod cli;
mod db;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Notification { subcommand } => match subcommand {
            cli::NotificationSubcommand::Server => {
                server::server().await.unwrap();
                Ok(())
            }
            cli::NotificationSubcommand::Query {
                query,
                json: _,
                raw: _,
            } => {
                let (sql, params) = db::parser::parse_query(&query).unwrap();
                // println!("{sql}");
                let mut database = db::DB::default().await;
                let notifications = database.get_notification(&sql, &params).await.unwrap();

                println!("{}", serde_json::to_string(&notifications).unwrap());
                Ok(())
            }
            cli::NotificationSubcommand::Close { notification_id } => {
                let mut database = db::DB::default().await;
                database.close_notification(notification_id).await;
                Ok(())
            }
        },
    }
}
