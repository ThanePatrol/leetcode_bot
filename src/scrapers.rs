use std::process::Command;
use thirtyfour::error::WebDriverResult;
use thirtyfour::{By, DesiredCapabilities, WebDriver};

///Everything in this file should be run in a installation that has chromedriver already installed
/// This function simply scrapes neetcode, storing all the questions that are not premium into a sqlite DB
pub async fn scrape_neetcode() -> WebDriverResult<()> {

    let capabilities = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", capabilities).await?;

    driver.goto("https://neetcode.io/practice").await?;

    let table = driver.find(
        By::XPath("/html/body/app-root/app-pattern-table-list/div/div[2]/div[4]/app-table/div/table/tbody")
    ).await?;
    println!("here");
    for link in table.find_all(By::Tag("a")).await? {
        println!("{:?}", link);
    }



    Ok(())
}

/// Assumes .env has already been loaded
/// web driver will exit once the program has finished
///
/// //todo - need to spawn a child process, currently the command is blocking output
/// either spawn in detached mode and cleanup later using commands or find a rust way to do it
pub fn init_webdriver() {
    let driver_path = std::env::var("WEB_DRIVER_PATH")
        .expect("Error reading token from .env");

    let output = Command::new(driver_path)

        .output()
        .expect("Failed to launch chromedriver");
    println!("{:?}", output)
}
