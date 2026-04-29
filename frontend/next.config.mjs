/** @type {import('next').NextConfig} */
const config = {
  webpack(cfg) {
    cfg.experiments = { ...cfg.experiments, asyncWebAssembly: true };
    return cfg;
  },
};

export default config;