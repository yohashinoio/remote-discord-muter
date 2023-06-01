// Set environments
require("dotenv").config();

const drpc = require("discord-rpc");
const express = require("express");

const env = process.env;

const client_id = env.CLIENT_ID;
const client_secret = env.CLIENT_SECRET;

const drpc_client = new drpc.Client({ transport: "ipc" });

function serve() {
    const app = express();
    const port = 3000;

    app.get("/", (_, res) => {
        drpc_client
            .getVoiceSettings()
            .then((e) => {
                return drpc_client.setVoiceSettings({ mute: !e.mute });
            })
            .then(() => {
                res.sendStatus(200);
            });
    });

    app.listen(port, "0.0.0.0", () => {
        console.log(`Listening at http://localhost:${port}`);
    });
}

drpc_client.on("ready", () => {
    serve();
});

drpc_client
    .login({
        clientId: client_id,
        clientSecret: client_secret,
        scopes: ["rpc"],
        redirectUri: env.REDIRECT_URI,
    })
    .catch(console.error);
