import axios from "axios";
import type React from "react";

type Props = {
  text: string;
  to: string;
};

export const PostButton: React.FC<Props> = ({ text, to }) => {
  function onClick() {
    console.log(`Send POST request to ${to}`);
    axios.post(to).catch((e) => console.log(e));
  }

  return <button onClick={onClick}>{text}</button>;
};
