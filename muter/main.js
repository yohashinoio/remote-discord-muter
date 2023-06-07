// Set environments
require("dotenv").config();

const DRPC = require("discord-rpc");

const dclient = new DRPC.Client({ transport: "ipc" });

dclient.on("ready", () => {
    const WebSocket = require("ws");

    const uri = `ws://${process.env.SERVER_IP}:3000/watch`;

    console.log(`Connecting to ${uri}`);

    const ws = new WebSocket(uri);

    ws.onopen = () => {
        console.log("Connection opened");

        ws.onclose = () => {
            console.log("Connection closed");
        };

        ws.onmessage = (msg) => {
            if (msg.data === "mute") {
                // Mute
                dclient.setVoiceSettings({ mute: true });
            } else if (msg.data === "unmute") {
                // Unmute
                dclient.setVoiceSettings({ mute: false });
            }
        };
    };
});

dclient
    .login({
        clientId: process.env.CLIENT_ID,
        clientSecret: process.env.CLIENT_SECRET,
        scopes: ["rpc"],
        redirectUri: process.env.REDIRECT_URI,
    })
    .catch(console.error);
