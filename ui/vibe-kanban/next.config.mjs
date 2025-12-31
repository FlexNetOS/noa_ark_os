/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  typedRoutes: true,
  experimental: {
    externalDir: true,
  },
  transpilePackages: ["@noa-ark/shared-ui"],
  typescript: {
    ignoreBuildErrors: true,
  },
};

export default nextConfig;
