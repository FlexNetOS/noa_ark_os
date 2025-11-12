/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  experimental: {
    typedRoutes: true,
    externalDir: true,
  },
  transpilePackages: ["@noa-ark/shared-ui"],
};

export default nextConfig;
