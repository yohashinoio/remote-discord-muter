require("dotenv").config();

import WebSocket from "ws";
import DRPC from "discord-rpc";
import axios from "axios";

const dc = new DRPC.Client({ transport: "ipc" });

dc.on("ready", () => {
    const ws_uri = `wss://${process.env.SERVER_HOSTNAME}/watch/${dc.user?.username}/${dc.user?.id}/${dc.user?.avatar}`;

    console.log(`Connecting to ${ws_uri}`);

    const ws = new WebSocket(ws_uri);

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
            let data = msg.data.toString();

            const set_mute_setting = (mute: boolean) => {
                dc.getVoiceSettings()
                    .then((s) => {
                        s.mute = mute;
                        s.input = undefined;
                        s.output = undefined;
                        dc.setVoiceSettings(s);
                    })
                    .catch(() => {
                        console.log("Failed to " + mute ? "mute" : "unmute");
                    });
            };

            if (data === "mute") {
                set_mute_setting(true);
            } else if (data === "unmute") {
                set_mute_setting(false);
            } else if (data.startsWith("GET STATUS MUTE")) {
                // Request to get mute status
                const splited: string[] = data.split(" ");
                let resp_id = splited[splited.length - 1];

                dc.getVoiceSettings()
                    .then((s) => {
                        ws.send(`RESP ${resp_id} ${s.mute}`);
                    })
                    .catch(() => {
                        ws.send(`RESP ${resp_id} ERR`);
                    });
            }
        };
    };
});

const client_id = process.env.CLIENT_ID;
const client_secret = process.env.CLIENT_SECRET;
const redirect_uri = process.env.REDIRECT_URI;

if (client_id === undefined) {
    throw new Error("CLIENT_ID environment variable is not set");
}

if (client_secret === undefined) {
    throw new Error("CLIENT_SECRET environment variable is not set");
}

if (redirect_uri === undefined) {
    throw new Error("REDIRECT_URI environment variable is not set");
}

dc.login({
    clientId: client_id,
    clientSecret: client_secret,
    scopes: ["rpc"],
    redirectUri: redirect_uri,
}).catch(console.error);
