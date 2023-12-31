import React from "react";
import { IconButton, Skeleton } from "@chakra-ui/react";
import axios from "axios";
import { BsMicMute, BsMic } from "react-icons/bs";
import { CurrentUserContext } from "@/contexts/user";

const get_ws_origin = () => {
  let loc = window.location;
  let res: string;

  if (loc.protocol === "https:") {
    res = "wss:";
  } else {
    res = "ws:";
  }

  res += "//" + loc.host;

  return res;
};

export const ToggleMuteButton: React.FC = () => {
  const [mute_setting, setMuteSetting] = React.useState<boolean | null>(null);

  const { current_user } = React.useContext(CurrentUserContext);

  const toggle_mute = () => {
    const api = `/api/${mute_setting ? "unmute" : "mute"}/${
      current_user?.uuid
    }`;

    console.log(`POST to ${api}`);

    axios.post(api);
  };

  React.useEffect(() => {
    if (current_user !== undefined) {
      const api = `${get_ws_origin()}/api/watch/setting/mute/${
        current_user?.uuid
      }`;

      console.log(`Try websocket connection to ${api}`);

      const websocket = new WebSocket(api);

      websocket.addEventListener("open", () =>
        console.log(`Websocket to ${api} opened`)
      );

      websocket.addEventListener("close", () =>
        console.log(`Websocket to ${api} closed`)
      );

      const onMessage = (event: MessageEvent<string>) => {
        if (event.data === "muted") {
          console.log("Muted");
          setMuteSetting(true);
        } else if (event.data === "unmuted") {
          console.log("Unmuted");
          setMuteSetting(false);
        }
      };
      websocket.addEventListener("message", onMessage);

      return () => {
        websocket.close();
        websocket.removeEventListener("message", onMessage);
      };
    }
  }, [current_user]);

  if (current_user === undefined || mute_setting === null) {
    return <Skeleton h={10} w={10} />;
  }

  return (
    <>
      {mute_setting ? (
        <IconButton
          onClick={toggle_mute}
          aria-label="Toggle mute"
          icon={<BsMicMute size={"24px"} />}
        />
      ) : (
        <IconButton
          onClick={toggle_mute}
          aria-label="Toggle mute"
          icon={<BsMic size={"24px"} />}
        />
      )}
    </>
  );
};
