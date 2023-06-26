# Leetcdoe bot
Discord bot for pinging people with daily leetcode questions.
The aim is to allow discord groups to easily have a daily question posted from a curated 
list of questions, either randomly or in a queue. 

## Quickstart
You will need to create roles for people to be notified of the questions. 

The preferred method of installation is via a docker-compose file. 
An example file might look like this:
```yaml
version: 3
# rest of file here
```
Note that the database file will need to be downloaded from this repository separately.

//todo - check this
You will also need to provide a `.env` file in the same directory as the database  
An example `.env` file would look like:
```dotenv
# Where xxx is your bot token
BOT_TOKEN=xxx

# This path assumes the database is in the same directory as the binary
# todo - check this
DATABASE_URL="sqlite://./leetcode_questions.db"

# Where xxx is your discord channel you use for posting questions
QUESTION_CHANNEL_ID=xxx

# Where xxx is your discord channel used for posting bot commands
# ideally this is the only use of the channel
COMMAND_CHANNEL_ID=xxx

# Where xxx is your bot user id
BOT_USER_ID=xxx

# Where xxx is your role id for specific questions
# if you don't want multiple roles, assign them to the same value
EASY_ROLE_ID=xxx
MED_ROLE_ID=xxx
HARD_ROLE_ID=xxx

# Customize your announcement text here
ANNOUNCEMENT_TEXT="The daily question is: "

# UTC Time to post the daily question, specified in 24 hour format
# eg 10:00:00 is 10AM UTC 
TIME_TO_POST="10:00:00"

```

## Usage
Currently, the supported commands are `push` and `pop`

### Push

`push..` - this adds a question to the queue. To specify which question to add you must provide the url
or the question number. eg: `push..1` or `push..https://leetcode.com/problems/two-sum/`
to push Two Sum into the queue

Note the url format.

### Pop
`pop` - this forces the bot to make a post with the next question in the queue.

If the queue is empty, a random question will be returned. 
Only people who have been subscribed to the question difficulty will be notified.

### View
`view` - this shows the details of the questions currently in the queue

## Difficulty
Notifications will only be sent to those who have signed up for the role. 
The intended use for roles are to allow notifications based upon question difficulty
(Easy, Medium, Hard).

If you don't want to use the roles and want everyone to be notified of every
question, simply set the same role id for all roles in the environment file

## Question lists
Currently, the two supported question lists are Neetcode 150 and Blind 75.
If you are interested in other lists, please open an issue. 

## Feature requests and support
If there are additional features you think would be beneficial,
or issues with installation please open an issue.

## Premium questions
Premium questions are not currently supported. 