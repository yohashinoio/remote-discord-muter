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
  SkeletonCircle,
  SkeletonText,
  Text,
} from "@chakra-ui/react";
import NextImage from "next/image";
import { fetcher } from "@/utils/fetcher";
import { ChevronDownIcon } from "@chakra-ui/icons";
import { DiscordUser } from "@/types/user";
import { CurrentUserContext } from "@/contexts/user";

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
  let get_watchers_api = `/api/watchers`;
  const { data, error, isLoading } = useSWR<DiscordUser[]>(
    get_watchers_api,
    (url: string) => {
      console.log(`GET to ${get_watchers_api}`);
      return fetcher(url);
    }
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
