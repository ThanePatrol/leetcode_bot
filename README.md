# Leetcdoe bot
Discord bot for pinging people with daily leetcode questions.
The aim is to allow discord groups to easily have a daily question posted from a curated 
list of questions, either randomly or in a queue. 


## Usage
Currently, the supported commands are:

### Push

`push..` - this adds a question to the queue. To specify which question to add you must provide the url
or the question number. eg: `push..1` or `push..https://leetcode.com/problems/two-sum/`
to push Two Sum into the queue

Note the url format.

### Pop
`pop` - this forces the bot to make a post with the next question in the queue.

If the queue is empty, a random question will be returned. 
Only people who have been subscribed to the question difficulty will be notified.

## Difficulty
Notifications will only be sent to those who have signed up for the role. 
The intended use for roles are to allow notifications based upon question difficulty
(Easy, Medium, Hard).

If you don't want to use the roles and want everyone to be notified of every
question, simply set the same role id for all roles in the environment file

## Question lists
Currently, the two supported question lists are Neetcode 150 and Blind 75.
If you are interested in other lists, please open an issue. 

## Premium questions
Premium questions are not currently supported. 