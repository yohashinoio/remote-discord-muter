const to_bool_from_env = (
  s: string | undefined
): boolean | undefined | null => {
  if (s === undefined) return undefined;

  if (s == "true") return true;

  if (s == "false") return false;

  return null;
};

const http_scheme = (() => {
  const ssl = to_bool_from_env(process.env.NEXT_PUBLIC_SSL);

  if (ssl === null)
    throw new Error("NEXT_PUBLIC_SSL must be set to a boolean value");

  if (ssl == undefined) return "https";

  return ssl ? "https" : "http";
})();

const websocket_scheme = (() => {
  const ssl = to_bool_from_env(process.env.NEXT_PUBLIC_SSL);

  if (ssl === null)
    throw new Error("NEXT_PUBLIC_SSL must be set to a boolean value");

  if (ssl == undefined) return "wss";

  return ssl ? "wss" : "ws";
})();

export const server_origin_http = `${http_scheme}://${process.env.NEXT_PUBLIC_SRV_HOST_PORT}`;

export const server_origin_websocket = `${websocket_scheme}://${process.env.NEXT_PUBLIC_SRV_HOST_PORT}`;
