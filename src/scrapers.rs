use std::{process, thread};
use std::process::Command;
use std::time::Duration;
use thirtyfour::error::WebDriverResult;
use thirtyfour::{By, DesiredCapabilities, WebDriver};
use tokio::join;
use crate::leetcode::Leetcode;


///Everything in this file should be run in a installation that has chromedriver already installed
/// This function simply scrapes neetcode, storing all the questions that are not premium into a sqlite DB
pub async fn scrape_neetcode() -> WebDriverResult<Vec<Leetcode>> {
    let mut questions = Vec::new();

    let capabilities = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", capabilities).await?;

    driver.goto("https://neetcode.io/practice").await?;

    //change from neetcode 150 to neetcode all
    let view_all_link = driver.find(By::XPath("/html/body/app-root/app-pattern-table-list/div/div[2]/div[2]/ul/li[3]/a"))
        .await?;
    view_all_link.click().await?;

    //change from grouped by topic to all question view
    let list_view_link = driver.find(
        By::XPath("/html/body/app-root/app-pattern-table-list/div/div[2]/div[1]/div/button[1]")
    ).await?;
    list_view_link.click().await?;



    //there's 3 pages of questions, click through all of them,
    // not the most scalable solution but it works
    for _ in 0..3 {

        let table_of_questions = driver.find(
            By::XPath("/html/body/app-root/app-pattern-table-list/div/div[2]/div[4]/app-table/div/table/tbody")
        ).await?;

        // walk through the table, find links, create struct and push to vec
        for table_row in table_of_questions.find_all(By::Tag("tr")).await? {
            for table_division in table_row.find_all(By::Tag("td")).await? {
                match table_division.find(By::ClassName("table-text")).await {
                    Ok(anchor_tag) => {
                        let problem_name = anchor_tag.text().await?;
                        let problem_link = anchor_tag.attr("href").await?
                            .expect("No problem link found");

                        let problem = Leetcode::new(problem_name, problem_link);
                        questions.push(problem);
                    }
                    Err(_) => {}
                }
            }
        }

        //click on link for next page, this won't fail on the last page as it is still populated with a button
        let next_page = driver.find(
            By::XPath("/html/body/app-root/app-pattern-table-list/div/div[2]/div[4]/app-table/div/div/button[2]")
        ).await?;
        next_page.click().await?;

    }


    Ok(questions)
}

/// Assumes .env has already been loaded
/// web driver will exit once the program has finished
///
/// //todo - need to spawn a child process, currently the command is blocking output
/// either spawn in detached mode and cleanup later using commands or find a rust way to do it
pub fn init_webdriver() -> process::Child {
    let driver_path = std::env::var("WEB_DRIVER_PATH")
        .expect("Error reading token from .env");

    let child_proces = Command::new(driver_path)
        .spawn()
        .expect("Failed to launch chromedriver");
    child_proces
}
