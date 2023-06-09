#[allow(dead_code)]
pub mod scraper {
    use std::{process, thread};
    use std::process::Command;
    use std::time::Duration;
    use thirtyfour::error::WebDriverResult;
    use thirtyfour::{By, DesiredCapabilities, WebDriver};
    use thirtyfour::prelude::{ElementWaitable};
    use crate::leetcode::{Difficulty, Leetcode};

    /// Everything in this file should be run in a installation that has chromedriver already installed
    /// This function simply scrapes neetcode, storing all the questions into a sqlite DB
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

    pub async fn scrape_grind75() -> WebDriverResult<Vec<Leetcode>> {
        let mut questions = Vec::new();

        let capabilities = DesiredCapabilities::chrome();
        let driver = WebDriver::new("http://localhost:9515", capabilities).await?;

        driver.goto("https://www.techinterviewhandbook.org/grind75?hours=40&weeks=12&grouping=none").await?;

        // get main div that has all the questions
        let all_question_div = driver.find(By::XPath(
            "/html/body/div/main/div[1]/div/div/div[2]/div[2]/div/div/div[3]/div[4]"
        )).await?;


        // get the html of every element
        for id in 1..=169 {
            let x_path = format!("/html/body/div/main/div[1]/div/div/div[2]/div[2]/div/div/div[3]/div[4]/div[{}]", id);
            let child = all_question_div.find(By::XPath(&*x_path)).await?;

            let link_xpath = x_path + "/div[2]/div[1]/div/a";
            let link = child.find(By::XPath(&*link_xpath)).await?;

            let text = link.inner_html().await?;

            let link_url = link.attr("href").await?.unwrap();
            let question = Leetcode::new(text, link_url);
            questions.push(question);
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
        thread::sleep(Duration::from_millis(2000));

        // if we can't read question difficulty then it's a premium problem
        if let Ok(difficulty_div) = driver.find(By::XPath(
            "/html/body/div[1]/div/div/div/div/div/div[1]/div/div/div/div[2]/div/div/div[1]/div/div[2]/div[1]"
        )).await {
            difficulty_div.wait_until().displayed().await?;

            question.difficulty = Difficulty::new(difficulty_div.text().await?);

            // the span contains the number followed by a period then the problem name
            let problem_span = driver.find(By::XPath(
                "/html/body/div[1]/div/div/div/div/div/div[1]/div/div/div/div[2]/div/div/div[1]/div/div[1]/div[1]/div/span"
            ))
                .await
                .expect(&*format!("Error reading problem number from {}", question.url));
            problem_span.wait_until().displayed().await?;

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


            // get all topics
            let topic_div = driver.find(By::XPath(
                "//*[@id='qd-content']/div[1]/div/div/div/div[2]/div/div/div[last() - 1]/div/div[2]/div"
            ))
                .await
                .expect(&*format!("Error finding topics for {}", question.url));

            thread::sleep(Duration::from_millis(2000));

            let mut topics = Vec::new();


            for topic in topic_div.find_all(By::Tag("a")).await? {
                topics.push(topic.inner_html().await?);
            }

            question.categories = topics;
            Ok(question)
        } else {
            question.categories = vec!["premium_questions".to_string()];
            Ok(question)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scrapers;
    use scrapers::scraper::*;
    use super::*;

    //assumes chromedriver is already running and the total amount of neetcode questions is 434
    #[tokio::test]
    async fn test_all_questions_scraped_from_neetcode() {
        let env_file = dotenvy::dotenv().expect("Could not read .env file");
        let driver = init_webdriver();
        let questions = scrape_neetcode().await.unwrap();
        assert_eq!(questions.len(), 434);
    }
}

