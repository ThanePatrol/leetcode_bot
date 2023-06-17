use std::{process, thread};
use std::process::Command;
use std::time::Duration;
use thirtyfour::error::WebDriverResult;
use thirtyfour::{By, DesiredCapabilities, WebDriver};
use tokio::join;
use crate::leetcode::{Difficulty, Leetcode};


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
/// unless the the program has previously crashed or chromedriver was launched separately
pub fn init_webdriver() -> process::Child {
    let driver_path = std::env::var("WEB_DRIVER_PATH")
        .expect("Error reading token from .env");

    let child_proces = Command::new(driver_path)
        .spawn()
        .expect("Failed to launch chromedriver");
    child_proces
}

/// Goes to a single problem page, gets all information.
/// question parameter is assumed to have a the question name and url population
/// the question parameter is updated with information on the page
/// and returned at the end of the function
pub async fn get_problem_details(mut question: Leetcode) -> WebDriverResult<Leetcode> {
    let capabilities = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", capabilities).await?;

    let url = question.url.clone();
    driver.goto(url).await?;
    thread::sleep(Duration::from_millis(1000));

    let difficulty_div = driver.find(By::XPath(
        "/html/body/div[1]/div/div/div/div/div/div[1]/div/div/div/div[2]/div/div/div[1]/div/div[2]/div[1]"
    ))
        .await?;

    question.difficulty = Difficulty::new(difficulty_div.text().await?);

    // the span contains the number followed by a period then the problem name
    let problem_span = driver.find(By::XPath(
        "/html/body/div[1]/div/div/div/div/div/div[1]/div/div/div/div[2]/div/div/div[1]/div/div[1]/div[1]/div/span"
    ))
        .await?;

    // get the span string, collect all the digits into a string then parse into number
    let problem_number = problem_span
        .text()
        .await?
        .chars()
        .take_while(|&ch| ch.is_numeric())
        .collect::<String>()
        .parse::<u32>()
        .expect("Error parsing question number");

    question.number = problem_number;

    thread::sleep(Duration::from_millis(2000));


    // get all topics
    let topic_div = driver.find(By::XPath(
        "/html/body/div[1]/div/div/div/div/div/div[1]/div/div/div/div[2]/div/div/div[7]/div/div[2]/div"
    ))
        .await?;


    let mut topics = Vec::new();
    for topic in topic_div.find_all(By::Tag("a")).await? {
        topics.push(topic.inner_html().await?);
    }

    question.categories = topics;

    Ok(question)
}
