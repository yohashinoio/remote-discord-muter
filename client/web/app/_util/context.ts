import React from "react";
import { DiscordUser } from "./types";

export const CurrentUserContext = React.createContext(
  {} as {
    current_user: DiscordUser | undefined;
    setCurrentUser: React.Dispatch<
      React.SetStateAction<DiscordUser | undefined>
    >;
  }
);
