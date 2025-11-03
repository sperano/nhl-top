use anyhow::Result;
use nhl_api::Client;

pub async fn run(client: &Client) -> Result<()> {
    let franchises = client.franchises().await?;

    println!("\nNHL Franchises");
    println!("==============\n");

    println!("{:<5} {:<40} {:<25} {}", "ID", "Full Name", "Common Name", "Place Name");
    println!("{}", "─".repeat(100));

    for franchise in franchises {
        println!(
            "{:<5} {:<40} {:<25} {}",
            franchise.id,
            franchise.full_name,
            franchise.team_common_name,
            franchise.team_place_name
        );
    }

    println!();
    Ok(())
}
