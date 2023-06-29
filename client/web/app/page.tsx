"use client";

import React from "react";
import { Users } from "./_components/Users";
import { Center, Skeleton, Stack } from "@chakra-ui/react";
import { ToggleMuteButton } from "./_components/ToggleMuteButton";
import { CurrentUserContext } from "./_util/context";
import { DiscordUser } from "./_util/types";

export default function Home() {
  const [current_user, setCurrentUser] = React.useState<DiscordUser>();

  return (
    <CurrentUserContext.Provider value={{ current_user, setCurrentUser }}>
      <Center h={"100vh"}>
        <Stack>
          <Users />
          <Center>
            <ToggleMuteButton />
          </Center>
        </Stack>
      </Center>
    </CurrentUserContext.Provider>
  );
}
