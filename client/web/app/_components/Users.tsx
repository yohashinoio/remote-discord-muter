"use client";

import React from "react";
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
  SkeletonCircle,
  SkeletonText,
  Text,
} from "@chakra-ui/react";
import { ChevronDownIcon } from "@chakra-ui/icons";
import NextImage from "next/image";
import { CurrentUserContext } from "../_util/context";
import { fetcher } from "../_util/fetcher";
import { DiscordUser } from "../_util/types";
import { server_origin_http } from "../_util/server";

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

const DummyUserCard: React.FC = () => {
  return (
    <Flex>
      <Center>
        <SkeletonCircle size={"32px"} />
      </Center>
      <Center ml={2}>
        <SkeletonText width={"28"} noOfLines={1} skeletonHeight={3} />
      </Center>
    </Flex>
  );
};

export const Users: React.FC = () => {
  const { data, error, isLoading } = useSWR<DiscordUser[]>(
    `${server_origin_http}/watchers`,
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
    <Menu>
      <MenuButton as={Button} rightIcon={<ChevronDownIcon />}>
        {isLoading ? (
          <DummyUserCard />
        ) : current_user ? (
          <UserCard {...current_user} />
        ) : (
          <DummyUserCard />
        )}
      </MenuButton>
      <MenuList>
        {data?.map((e) => (
          <MenuItem onClick={() => setCurrentUser(e)} key={e.user_id}>
            <UserCard {...e} />
          </MenuItem>
        ))}
      </MenuList>
    </Menu>
  );
};
