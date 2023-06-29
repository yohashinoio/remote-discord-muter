require("dotenv").config();

import WebSocket from "ws";
import DRPC, { VoiceSettings } from "discord-rpc";
import axios from "axios";

const dc = new DRPC.Client({ transport: "ipc" });

dc.on("ready", () => {
    dc.subscribe("VOICE_SETTINGS_UPDATE", {});

    const watch_api = `${process.env.WEBSOCKET_SCHEME}://${process.env.SERVER_HOST_PORT}/watch/${dc.user?.username}/${dc.user?.id}/${dc.user?.avatar}`;

    console.log(`Connecting to ${watch_api}`);

    const ws = new WebSocket(watch_api);

    ws.onopen = () => {
        console.log("Connection opened");

        // Measures to be taken for servers that stop without regular access
        setInterval(() => {
            axios.get(`http://${process.env.SERVER_HOSTNAME}/ok`);
        }, 300000 /* 5 minutes */);

        dc.on("VOICE_SETTINGS_UPDATE", (vs: VoiceSettings) => {
            if (vs.mute) {
                ws.send("muted");
            } else {
                ws.send("unmuted");
            }
        });

        ws.onclose = () => {
            console.log("Connection closed");
        };

        ws.onmessage = (msg) => {
            let data = msg.data.toString();

            const set_mute_setting = (mute: boolean) => {
                dc.getVoiceSettings()
                    .then((vs) => {
                        vs.mute = mute;
                        vs.input = undefined;
                        vs.output = undefined;
                        dc.setVoiceSettings(vs);
                    })
                    .catch(() => {
                        console.log("Failed to " + mute ? "mute" : "unmute");
                    });
            };

            if (data === "mute") {
                set_mute_setting(true);
            } else if (data === "unmute") {
                set_mute_setting(false);
            } else if (data.startsWith("GET SETTING MUTE")) {
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
