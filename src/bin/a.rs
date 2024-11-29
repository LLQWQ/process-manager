use dialoguer::{theme::ColorfulTheme, Confirm};

fn main() {
    match Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you really really really really really want to continue?")
        .default(true)
        .wait_for_newline(true)
        .interact_opt()
        .unwrap()
    {
        Some(true) => println!("Looks like you want to continue"),
        Some(false) => println!("nevermind then :("),
        None => println!("Ok, we can start over later"),
    }
}
