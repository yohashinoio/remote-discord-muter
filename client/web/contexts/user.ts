import { DiscordUser } from "@/types/user";
import React from "react";

export const CurrentUserContext = React.createContext(
  {} as {
    current_user: DiscordUser | undefined;
    setCurrentUser: React.Dispatch<
      React.SetStateAction<DiscordUser | undefined>
    >;
  }
);
