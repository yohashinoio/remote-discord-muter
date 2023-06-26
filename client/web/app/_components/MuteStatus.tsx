"use client";

import React from "react";
import { CurrentUserContext, fetcher } from "../page";
import useSWR, { mutate } from "swr";
import { Box, IconButton, Spinner } from "@chakra-ui/react";
import { BsMic, BsMicMute } from "react-icons/bs";
import axios from "axios";

type MuteStatus = {
  mute: boolean;
};

export const MuteStatus: React.FC = () => {
  const { current_user } = React.useContext(CurrentUserContext);

  const mute_status_api = `${process.env.NEXT_PUBLIC_URI_SCHEME}://${process.env.NEXT_PUBLIC_SERVER_HOSTNAME}/status/mute/${current_user?.uuid}`;

  const { data, error, isLoading, isValidating } = useSWR<MuteStatus>(
    mute_status_api,
    fetcher
  );

  if (error) {
    return null;
  }

  if (isLoading) {
    return <Spinner />;
  }

  const toggle_mute = () => {
    let api = `${process.env.NEXT_PUBLIC_URI_SCHEME}://${
      process.env.NEXT_PUBLIC_SERVER_HOSTNAME
    }/${data?.mute ? "unmute" : "mute"}/${current_user?.uuid}`;

    axios.post(api);
  };

  return (
    <>
      {isValidating ? (
        <Spinner />
      ) : data?.mute ? (
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
