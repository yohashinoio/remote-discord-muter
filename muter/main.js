// Set environments
require("dotenv").config();

const { default: axios } = require("axios");
const DRPC = require("discord-rpc");

const dclient = new DRPC.Client({ transport: "ipc" });

function connect() {
    const WebSocket = require("ws");

    const uri = `wss://${process.env.SERVER_HOSTNAME}/watch`;

    console.log(`Connecting to ${uri}`);

    const ws = new WebSocket(uri);

    ws.onopen = () => {
        console.log("Connection opened");

        // Measures to be taken for servers that stop without regular access
        setInterval(() => {
            axios.get(`http://${process.env.SERVER_HOSTNAME}/ok`);
        }, 300000 /* 5 minutes */);

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
}

dclient.on("ready", () => {
    connect();
});

dclient
    .login({
        clientId: process.env.CLIENT_ID,
        clientSecret: process.env.CLIENT_SECRET,
        scopes: ["rpc"],
        redirectUri: process.env.REDIRECT_URI,
    })
    .catch(console.error);
