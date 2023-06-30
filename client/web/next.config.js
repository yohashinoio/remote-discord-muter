/** @type {import('next').NextConfig} */
const nextConfig = {
  images: {
    domains: ["cdn.discordapp.com"],
    unoptimized: true,
  },
  assetPrefix: "./",
  reactStrictMode: true,
};

module.exports = nextConfig;
