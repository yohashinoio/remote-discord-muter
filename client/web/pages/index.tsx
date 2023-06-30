import React from "react";
import { Center, Stack } from "@chakra-ui/react";
import Head from "next/head";
import { ToggleMuteButton } from "@/components/ToggleMuteButton";
import { Users } from "@/components/Users";
import { DiscordUser } from "@/types/user";
import { CurrentUserContext } from "@/contexts/user";

export default function Home() {
  const [current_user, setCurrentUser] = React.useState<DiscordUser>();

  return (
    <>
      <Head>
        <title>Remote Discord Muter</title>
        <meta name="description" content="Client for remote discord muter" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link rel="icon" href="/favicon.ico" />
      </Head>
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
    </>
  );
}
