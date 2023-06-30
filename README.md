# How to use?

## Only in a local network

If you just want to use this from a local network, it is easy.

1. Build and run the Dockerfile in the root
2. Copy .example.env in /muter and rename it to .env
3. Fill in the .env ([details](#dc))
4. Run 'npm i'
5. Run 'npx ts-node main.ts'
6. Access to localhost:8080 with a web browser

## Even from outside a local network

To make it available from outside local networks, a server is required.

Here's how to deploy with [render.com](https://render.com/). (free)

1. Fork this repository
2. Create an account on render.com
3. Go to your dashboard
4. Go to [the page](https://dashboard.render.com/select-repo?type=web) to create a new web service

If you cannot find the forked repository, configure your GitHub account.

5. After confirming that runtime is docker, create

From here, please work on a computer that is actually running Discord Desktop.

6. Copy .example.env in /muter and rename it to .env
7. Fill in the .env ([details](#dc))
8. Run 'npm i'
9. Run 'npx ts-node main.ts'
10. Access to the deployed server (e.g. xxx.onrender.com)

<a id="dc"></a>

# Setting environment variables

## Discord related

The environment variables DISCORD_CLIENT_ID, DISCORD_CLIENT_SECRET and DISCORD_REDIRECT_URI must be set to run muter.

These can be obtained via the [Discord Developer Portal](https://discord.com/developers/applications).

## Others

SERVER_HOST_PORT and WEBSOCKET_SCHEME need not be changed from .example.env if used in a local network.

If used from outside local networks, SERVER_HOST_PORT should be the hostname of the server you deployed to.

WEBSOCKET_SCHEME can basically be changed to wss.
