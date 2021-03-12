# Slack Echo Bot

Based on [slack-morphism-rust example](https://github.com/abdolence/slack-morphism-rust/tree/master/src/examples).

Once installed this this app will reply to any direct message with an identical response.

## How to install an app into your workspace

This is current as of 2021-03-12.

1. [Create an App](https://api.slack.com/apps?new_app)
2. Add the `chat:write` and `im:read` OAuth Scopes under *OAuth & Permissions*
3. *Install App* into your workspace
4. Store the *Bot User OAuth token* (visible under *Install App* or *OAuth & Permissions*) in the `SLACK_BOT_TOKEN` environment variable
5. Store the values from *Basic Information*/*App Credentials* in `SLACK_CLIENT_ID`, `SLACK_CLIENT_SECRET`
6. Store empty values in `SLACK_BOT_SCOPE` and `SLACK_REDIRECT_HOST`
7. Launch the server with all the environment variables set
8. *Enable Events* in *Event Subscriptions* using `http[s]://YOUR_DOMAIN_OR_IP/push`
9. *Subscribe to bot events* `message.im` or `message.channels` in *Event Subscriptions* (Watch for the *Save Changes* banner on the bottom of your screen, its easy to miss. Until you click *Save Changes* you will not receive events).

You should be able to DM the bot now and it will repeat whatever you say back. Be sure to "save" on some pages (like *OAuth & Permissions*)
