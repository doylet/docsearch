import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: 'standalone',
  // Optimize for production
  compress: true,
  poweredByHeader: false,
};

export default nextConfig;
