"use client";

import React from "react";
import { DiscordUser, CurrentUserContext, fetcher } from "../page";
import useSWR from "swr";
import {
  Box,
  Button,
  Center,
  Flex,
  Menu,
  MenuButton,
  MenuItem,
  MenuList,
  Skeleton,
  Text,
} from "@chakra-ui/react";
import { ChevronDownIcon } from "@chakra-ui/icons";
import NextImage from "next/image";

const UserCard: React.FC<DiscordUser> = (props) => {
  const avatar_src = `https://cdn.discordapp.com/avatars/${props.user_id}/${props.avatar_id}.png`;

  return (
    <Flex>
      <Box borderRadius={"full"} overflow={"hidden"}>
        <NextImage
          width={32}
          height={32}
          alt={`${props.username}'s Profile Picture`}
          src={avatar_src}
          quality={100}
        />
      </Box>

      <Center ml={2}>
        <Text>{props.username}</Text>
      </Center>
    </Flex>
  );
};

export const Users: React.FC = () => {
  const { data, error, isLoading } = useSWR<DiscordUser[]>(
    `${process.env.NEXT_PUBLIC_URI_SCHEME}://${process.env.NEXT_PUBLIC_SERVER_HOSTNAME}/watchers`,
    fetcher
  );

  const { current_user, setCurrentUser } = React.useContext(CurrentUserContext);

  React.useEffect(() => {
    if (!isLoading) {
      setCurrentUser(data?.at(0));
    }
  }, [isLoading]);

  if (error) {
    return null;
  }

  return (
    <Skeleton isLoaded={!isLoading}>
      <Menu>
        <MenuButton as={Button} rightIcon={<ChevronDownIcon />}>
          {current_user && <UserCard {...current_user} />}
        </MenuButton>
        <MenuList>
          {data?.map((e) => (
            <MenuItem onClick={() => setCurrentUser(e)} key={e.user_id}>
              <UserCard {...e} />
            </MenuItem>
          ))}
        </MenuList>
      </Menu>
    </Skeleton>
  );
};
