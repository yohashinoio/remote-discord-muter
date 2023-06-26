"use client";

import axios from "axios";
import React from "react";
import { Users } from "./_components/Users";
import { Center, Stack } from "@chakra-ui/react";
import { MuteStatus } from "./_components/MuteStatus";

export const fetcher = (url: string) => axios.get(url).then((res) => res.data);

export type DiscordUser = {
  uuid: string;
  username: string;
  user_id: string;
  avatar_id: string;
};

export const CurrentUserContext = React.createContext(
  {} as {
    current_user: DiscordUser | undefined;
    setCurrentUser: React.Dispatch<
      React.SetStateAction<DiscordUser | undefined>
    >;
  }
);

export default function Home() {
  const [current_user, setCurrentUser] = React.useState<DiscordUser>();

  return (
    <CurrentUserContext.Provider value={{ current_user, setCurrentUser }}>
      <Center h={"100vh"}>
        <Stack>
          <Users />
          <Center>
            <MuteStatus />
          </Center>
        </Stack>
      </Center>
    </CurrentUserContext.Provider>
  );
}
